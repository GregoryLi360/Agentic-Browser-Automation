//! The broker orchestrator — the challenge-iterator authentication runtime. Composes a
//! credential source, a surface, an OTP generator, and a policy (trait objects, so any
//! impl slots in), plus optional verification and passkey satisfiers and any number of
//! flows. `authenticate` loops: detect the next challenge, satisfy it, repeat. Enforces
//! target policy and never returns secret values.

use crate::broker::{AuthOptions, AuthOutcome, Broker, BrokerError};
use crate::flow::{Flow, FlowKind, FlowOutcome};
use crate::model::{Credential, CredentialKind, Item, ItemSummary, Target};
use crate::passkey::PasskeyAuthenticator;
use crate::policy::TargetPolicy;
use crate::source::{Channel, OtpGenerator, PasswordManager, Status, VerificationSource};
use crate::surface::{Challenge, FieldKind, Surface};

/// Backstop against a surface whose challenge never clears (bad creds, undetected step).
const MAX_STEPS: usize = 16;

pub struct BrokerService {
    manager: Box<dyn PasswordManager>,
    surface: Box<dyn Surface>,
    otp: Box<dyn OtpGenerator>,
    policy: Box<dyn TargetPolicy>,
    verification: Option<Box<dyn VerificationSource>>,
    passkey: Option<Box<dyn PasskeyAuthenticator>>,
    flows: Vec<Box<dyn Flow>>,
}

/// Whether a satisfied challenge advanced, or parked on an out-of-band wait.
enum Step {
    Done,
    Pending,
}

impl BrokerService {
    pub fn new(
        manager: impl PasswordManager + 'static,
        surface: impl Surface + 'static,
        otp: impl OtpGenerator + 'static,
        policy: impl TargetPolicy + 'static,
    ) -> Self {
        BrokerService {
            manager: Box::new(manager),
            surface: Box::new(surface),
            otp: Box::new(otp),
            policy: Box::new(policy),
            verification: None,
            passkey: None,
            flows: Vec::new(),
        }
    }

    /// Register a delivered-code (email/SMS) source for `Challenge::OtpCode` when an item
    /// has no stored TOTP seed.
    pub fn with_verification(mut self, source: impl VerificationSource + 'static) -> Self {
        self.verification = Some(Box::new(source));
        self
    }

    /// Register a passkey authenticator for `Challenge::Passkey`.
    pub fn with_passkey(mut self, passkey: impl PasskeyAuthenticator + 'static) -> Self {
        self.passkey = Some(Box::new(passkey));
        self
    }

    /// Register a flow for `Challenge::Federated` (OAuth/SSO, magic link, ...). Additive.
    pub fn with_flow(mut self, flow: impl Flow + 'static) -> Self {
        self.flows.push(Box::new(flow));
        self
    }

    /// Resolve exactly one item for `target`. Callers authorize the target first.
    fn find_item(&self, target: &Target) -> Result<Item, BrokerError> {
        let mut hits: Vec<Item> =
            self.manager.items()?.into_iter().filter(|i| i.matches(target)).collect();
        match hits.len() {
            0 => Err(BrokerError::NotFound(target.clone())),
            1 => Ok(hits.remove(0)),
            _ => Err(BrokerError::Ambiguous {
                target: target.clone(),
                candidates: hits.iter().map(|i| i.name.clone()).collect::<Vec<_>>().join(", "),
            }),
        }
    }

    /// Satisfy one detected challenge. `item` is the resolved login (absent only when none
    /// matched — fine for federated/approval, an error for credential challenges).
    fn satisfy(
        &self,
        challenge: &Challenge,
        target: &Target,
        item: Option<&Item>,
        opts: &AuthOptions,
    ) -> Result<Step, BrokerError> {
        match challenge {
            Challenge::Password => {
                let item = item.ok_or_else(|| BrokerError::NotFound(target.clone()))?;
                let Some(Credential::BasicAuth { username, password }) =
                    item.credential(CredentialKind::BasicAuth)
                else {
                    return Err(BrokerError::MissingCredential {
                        item: item.name.clone(),
                        kind: "basic-auth",
                    });
                };
                let mut filled = false;
                if let Some(user) = username.as_deref().filter(|u| !u.is_empty()) {
                    filled |= self.surface.fill(FieldKind::Username, user)?;
                }
                filled |= self.surface.fill(FieldKind::Password, password.expose())?;
                if !filled {
                    return Err(BrokerError::NoLoginFields);
                }
                if opts.submit {
                    self.surface.submit()?;
                }
                Ok(Step::Done)
            }

            Challenge::OtpCode => {
                let item = item.ok_or_else(|| BrokerError::NotFound(target.clone()))?;
                let code = if let Some(Credential::Totp(totp)) =
                    item.credential(CredentialKind::Totp)
                {
                    self.otp.generate(totp)?
                } else if let Some(source) = &self.verification {
                    source.latest_code(Channel::Email)?
                } else {
                    return Err(BrokerError::MissingCredential {
                        item: item.name.clone(),
                        kind: "otp",
                    });
                };
                if !self.surface.fill(FieldKind::Otp, code.expose())? {
                    return Err(BrokerError::NoOtpField);
                }
                if opts.submit {
                    self.surface.submit()?;
                }
                Ok(Step::Done)
            }

            Challenge::Passkey => {
                let item = item.ok_or_else(|| BrokerError::NotFound(target.clone()))?;
                let Some(Credential::Passkey(passkey)) = item.credential(CredentialKind::Passkey)
                else {
                    return Err(BrokerError::MissingCredential {
                        item: item.name.clone(),
                        kind: "passkey",
                    });
                };
                let authenticator =
                    self.passkey.as_ref().ok_or(BrokerError::NoPasskeyAuthenticator)?;
                authenticator.assert(target, passkey)?;
                Ok(Step::Done)
            }

            Challenge::Federated { provider } => {
                let want = FlowKind::Sso { provider: provider.clone() };
                let flow = self
                    .flows
                    .iter()
                    .find(|f| f.supports(target) && f.kind() == want)
                    .ok_or_else(|| BrokerError::NoFlow(challenge.clone()))?;
                match flow.run(target)? {
                    FlowOutcome::SignedIn => Ok(Step::Done),
                    FlowOutcome::Pending { .. } => Ok(Step::Pending),
                }
            }

            // Push / number-match / QR: nothing the broker can type — park and let the
            // caller wait for the user to approve out of band.
            Challenge::Approval { .. } => Ok(Step::Pending),
        }
    }
}

impl Broker for BrokerService {
    fn authenticate(
        &self,
        requested: &Target,
        opts: AuthOptions,
    ) -> Result<AuthOutcome, BrokerError> {
        self.policy.authorize(requested)?;
        if !opts.skip_page_check {
            let observed = self.surface.target()?;
            self.policy.verify(&observed, requested)?;
        }
        // Lazy: federated/approval challenges need no stored item.
        let item = self.find_item(requested).ok();

        let mut steps: Vec<Challenge> = Vec::new();
        let mut last: Option<Challenge> = None;
        let mut repeats = 0;
        let mut forced = opts.force.clone();

        for _ in 0..MAX_STEPS {
            let challenge = match forced.take() {
                Some(challenge) => challenge,
                None => match self.surface.next_challenge()? {
                    Some(challenge) => challenge,
                    None => return Ok(AuthOutcome::Authenticated { steps }),
                },
            };

            if Some(&challenge) == last.as_ref() {
                repeats += 1;
                if repeats >= 2 {
                    return Err(BrokerError::Stuck(challenge));
                }
            } else {
                repeats = 0;
            }

            match self.satisfy(&challenge, requested, item.as_ref(), &opts)? {
                Step::Done => {}
                Step::Pending => {
                    return Ok(AuthOutcome::Pending { waiting_on: challenge, steps })
                }
            }
            last = Some(challenge.clone());
            steps.push(challenge);
        }
        Err(BrokerError::TooManySteps)
    }

    fn list(&self) -> Result<Vec<ItemSummary>, BrokerError> {
        Ok(self.manager.list()?)
    }

    fn status(&self) -> Result<Status, BrokerError> {
        Ok(self.manager.status()?)
    }

    fn unlock(&self) -> Result<(), BrokerError> {
        Ok(self.manager.unlock()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Algorithm, Credential, Item, Passkey, Secret, Totp};
    use crate::policy::PolicyError;
    use crate::source::{ManagerError, OtpError};
    use crate::surface::SurfaceError;
    use std::cell::RefCell;

    fn github() -> Target {
        Target::parse("github.com")
    }

    fn totp_seed() -> Totp {
        Totp {
            secret: Secret::new("JBSWY3DPEHPK3PXP"),
            period: 30,
            digits: 6,
            algorithm: Algorithm::Sha1,
            issuer: None,
        }
    }

    fn item_with(creds: Vec<Credential>) -> Item {
        Item { name: "GitHub".into(), urls: vec!["https://github.com".into()], credentials: creds }
    }

    struct FakeManager(Vec<Item>);
    impl PasswordManager for FakeManager {
        fn status(&self) -> Result<Status, ManagerError> {
            Ok(Status::Unlocked)
        }
        fn unlock(&self) -> Result<(), ManagerError> {
            Ok(())
        }
        fn items(&self) -> Result<Vec<Item>, ManagerError> {
            Ok(self.0.clone())
        }
    }

    struct FixedOtp;
    impl OtpGenerator for FixedOtp {
        fn generate(&self, _: &Totp) -> Result<Secret, OtpError> {
            Ok(Secret::new("123456"))
        }
    }

    struct AllowAll;
    impl TargetPolicy for AllowAll {
        fn authorize(&self, _: &Target) -> Result<(), PolicyError> {
            Ok(())
        }
        fn verify(&self, _: &Target, _: &Target) -> Result<(), PolicyError> {
            Ok(())
        }
    }

    type Recorder<T> = std::rc::Rc<RefCell<T>>;

    /// A surface that replays a scripted challenge sequence and records every fill/submit
    /// into shared handles the test can inspect after the surface is moved into the broker.
    struct ScriptedSurface {
        script: RefCell<Vec<Option<Challenge>>>,
        fills: Recorder<Vec<(FieldKind, String)>>,
        submits: Recorder<usize>,
    }
    impl ScriptedSurface {
        #[allow(clippy::type_complexity)]
        fn new(
            script: Vec<Option<Challenge>>,
        ) -> (Self, Recorder<Vec<(FieldKind, String)>>, Recorder<usize>) {
            let fills = Recorder::default();
            let submits = Recorder::default();
            let surface = ScriptedSurface {
                script: RefCell::new(script),
                fills: fills.clone(),
                submits: submits.clone(),
            };
            (surface, fills, submits)
        }
    }
    impl Surface for ScriptedSurface {
        fn target(&self) -> Result<Target, SurfaceError> {
            Ok(github())
        }
        fn next_challenge(&self) -> Result<Option<Challenge>, SurfaceError> {
            let mut script = self.script.borrow_mut();
            if script.is_empty() {
                Ok(None)
            } else {
                Ok(script.remove(0))
            }
        }
        fn fill(&self, field: FieldKind, value: &str) -> Result<bool, SurfaceError> {
            self.fills.borrow_mut().push((field, value.to_string()));
            Ok(true)
        }
        fn submit(&self) -> Result<(), SurfaceError> {
            *self.submits.borrow_mut() += 1;
            Ok(())
        }
    }

    fn broker(items: Vec<Item>, surface: ScriptedSurface) -> BrokerService {
        BrokerService::new(FakeManager(items), surface, FixedOtp, AllowAll)
    }

    #[test]
    fn runs_password_then_otp_in_order_to_completion() {
        let item = item_with(vec![
            Credential::BasicAuth { username: Some("me".into()), password: Secret::new("pw") },
            Credential::Totp(totp_seed()),
        ]);
        // The surface asks for a password page, then a TOTP page, then is satisfied.
        let (surface, fills, submits) =
            ScriptedSurface::new(vec![Some(Challenge::Password), Some(Challenge::OtpCode)]);
        let b = broker(vec![item], surface);

        let outcome = b
            .authenticate(&github(), AuthOptions { submit: true, ..Default::default() })
            .unwrap();

        assert_eq!(
            outcome,
            AuthOutcome::Authenticated { steps: vec![Challenge::Password, Challenge::OtpCode] }
        );
        assert_eq!(
            *fills.borrow(),
            vec![
                (FieldKind::Username, "me".to_string()),
                (FieldKind::Password, "pw".to_string()),
                (FieldKind::Otp, "123456".to_string()),
            ]
        );
        assert_eq!(*submits.borrow(), 2);
    }

    #[test]
    fn force_drives_a_modality_then_continues_by_detection() {
        let item = item_with(vec![
            Credential::BasicAuth { username: Some("me".into()), password: Secret::new("pw") },
            Credential::Totp(totp_seed()),
        ]);
        // Detection only sees the OTP page; the caller forces the password step first.
        let (surface, fills, _submits) = ScriptedSurface::new(vec![Some(Challenge::OtpCode)]);
        let b = broker(vec![item], surface);

        let outcome = b
            .authenticate(
                &github(),
                AuthOptions {
                    submit: true,
                    force: Some(Challenge::Password),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(
            outcome,
            AuthOutcome::Authenticated { steps: vec![Challenge::Password, Challenge::OtpCode] }
        );
        assert_eq!(fills.borrow().len(), 3); // user, pass, otp
    }

    #[test]
    fn approval_parks_as_pending() {
        let item = item_with(vec![Credential::BasicAuth {
            username: Some("me".into()),
            password: Secret::new("pw"),
        }]);
        let (surface, ..) =
            ScriptedSurface::new(vec![Some(Challenge::Approval { kind: "push".into() })]);
        let b = broker(vec![item], surface);

        let outcome = b.authenticate(&github(), AuthOptions::default()).unwrap();
        assert!(matches!(outcome, AuthOutcome::Pending { .. }));
    }

    #[test]
    fn passkey_challenge_without_authenticator_errors() {
        // Item HAS a passkey facet, so satisfy reaches the authenticator check — none is
        // registered on this broker.
        let item = item_with(vec![Credential::Passkey(Passkey {
            credential_id: vec![1, 2, 3],
            rp_id: "github.com".into(),
            user_handle: vec![4, 5, 6],
            key: Secret::new("key"),
        })]);
        let (surface, ..) = ScriptedSurface::new(vec![Some(Challenge::Passkey)]);
        let b = broker(vec![item], surface);

        let err = b.authenticate(&github(), AuthOptions::default()).unwrap_err();
        assert!(matches!(err, BrokerError::NoPasskeyAuthenticator));
    }

    #[test]
    fn empty_surface_is_immediately_authenticated() {
        let item = item_with(vec![Credential::BasicAuth {
            username: Some("me".into()),
            password: Secret::new("pw"),
        }]);
        let (surface, ..) = ScriptedSurface::new(vec![]); // nothing to satisfy
        let b = broker(vec![item], surface);

        let outcome = b.authenticate(&github(), AuthOptions::default()).unwrap();
        assert_eq!(outcome, AuthOutcome::Authenticated { steps: vec![] });
    }
}
