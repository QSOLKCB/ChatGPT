use crate::contracts::{Action, Approval, ContractError};
use crate::desktop;
use crate::executor;
use crate::obs::{self, ObsConnectionConfig};
use crate::policy::{self, Disposition};
use crate::receipts::{Receipt, ReceiptStatus};
use crate::secrets::SecretStore;

pub struct Runtime {
    execute_effects: bool,
    obs: Option<ObsRuntime>,
}

struct ObsRuntime {
    config: ObsConnectionConfig,
    secrets: SecretStore,
}

impl Runtime {
    pub fn simulated() -> Self {
        Self {
            execute_effects: false,
            obs: None,
        }
    }

    pub fn effectful() -> Self {
        Self {
            execute_effects: true,
            obs: None,
        }
    }

    pub fn effectful_with_obs(config: ObsConnectionConfig, secrets: SecretStore) -> Self {
        Self {
            execute_effects: true,
            obs: Some(ObsRuntime { config, secrets }),
        }
    }

    pub fn run(&self, action: &Action, approval: Option<&Approval>) -> Result<Receipt, ContractError> {
        let decision = policy::evaluate(action);
        match decision.disposition {
            Disposition::Deny => receipt_without_evidence(
                action,
                decision.disposition,
                ReceiptStatus::Denied,
                Some(decision.code),
            ),
            Disposition::ApprovalRequired => {
                if !approval.is_some_and(|record| record.permits(action)) {
                    return receipt_without_evidence(
                        action,
                        decision.disposition,
                        ReceiptStatus::ApprovalRequired,
                        Some("approval_missing_or_mismatched"),
                    );
                }
                self.execute_or_simulate(action, decision.disposition)
            }
            Disposition::Allow => self.execute_or_simulate(action, decision.disposition),
        }
    }

    fn execute_or_simulate(
        &self,
        action: &Action,
        decision: Disposition,
    ) -> Result<Receipt, ContractError> {
        if !self.execute_effects {
            return receipt_without_evidence(action, decision, ReceiptStatus::Simulated, None);
        }

        if action.kind() == "screen.capture" {
            return self.execute_desktop_capture(action, decision);
        }

        if action.kind().starts_with("obs.") {
            return self.execute_obs(action, decision);
        }

        match executor::execute(action) {
            Ok(evidence) => {
                let status = if evidence.exit_code == Some(0) {
                    ReceiptStatus::Completed
                } else {
                    ReceiptStatus::Failed
                };
                Receipt::new(action, decision, status, Some(evidence), None)
            }
            Err(executor::ExecutionError::Unsupported) => Receipt::new(
                action,
                decision,
                ReceiptStatus::Unsupported,
                None,
                Some("executor_unsupported"),
            ),
            Err(executor::ExecutionError::CredentialsUnsupported) => Receipt::new(
                action,
                decision,
                ReceiptStatus::Unsupported,
                None,
                Some("credential_injection_unavailable"),
            ),
            Err(executor::ExecutionError::InvalidArgv) => Receipt::new(
                action,
                decision,
                ReceiptStatus::Denied,
                None,
                Some("invalid_argv"),
            ),
            Err(executor::ExecutionError::SpawnFailed) => Receipt::new(
                action,
                decision,
                ReceiptStatus::Failed,
                None,
                Some("spawn_failed"),
            ),
        }
    }

    fn execute_desktop_capture(
        &self,
        action: &Action,
        decision: Disposition,
    ) -> Result<Receipt, ContractError> {
        match desktop::capture_live() {
            Ok(evidence) => Receipt::new_desktop(
                action,
                decision,
                ReceiptStatus::Completed,
                Some(evidence),
                None,
            ),
            Err(desktop::DesktopError::PortalDenied) => Receipt::new_desktop(
                action,
                decision,
                ReceiptStatus::Denied,
                None,
                Some("desktop_capture_denied"),
            ),
            Err(desktop::DesktopError::InvalidPortalUri | desktop::DesktopError::InvalidPng) => {
                Receipt::new_desktop(
                    action,
                    decision,
                    ReceiptStatus::Failed,
                    None,
                    Some("desktop_capture_invalid_artifact"),
                )
            }
            Err(desktop::DesktopError::ScreenshotTooLarge) => Receipt::new_desktop(
                action,
                decision,
                ReceiptStatus::Failed,
                None,
                Some("desktop_capture_too_large"),
            ),
            Err(
                desktop::DesktopError::RuntimeUnavailable
                | desktop::DesktopError::PortalFailed
                | desktop::DesktopError::ScreenshotOpenFailed,
            ) => Receipt::new_desktop(
                action,
                decision,
                ReceiptStatus::Failed,
                None,
                Some("desktop_capture_failed"),
            ),
        }
    }

    fn execute_obs(&self, action: &Action, decision: Disposition) -> Result<Receipt, ContractError> {
        let Some(obs_runtime) = &self.obs else {
            return Receipt::new_obs(
                action,
                decision,
                ReceiptStatus::Unsupported,
                None,
                Some("obs_not_configured"),
            );
        };

        match obs::execute(action, &obs_runtime.config, &obs_runtime.secrets) {
            Ok(evidence) => Receipt::new_obs(
                action,
                decision,
                ReceiptStatus::Completed,
                Some(evidence),
                None,
            ),
            Err(obs::ObsError::UnsupportedAction | obs::ObsError::UnsupportedRequest) => {
                Receipt::new_obs(
                    action,
                    decision,
                    ReceiptStatus::Unsupported,
                    None,
                    Some("obs_request_unsupported"),
                )
            }
            Err(obs::ObsError::InvalidArguments | obs::ObsError::CredentialBindingMismatch) => {
                Receipt::new_obs(
                    action,
                    decision,
                    ReceiptStatus::Denied,
                    None,
                    Some("obs_invalid_or_unbound_action"),
                )
            }
            Err(obs::ObsError::MissingCredential) => Receipt::new_obs(
                action,
                decision,
                ReceiptStatus::Failed,
                None,
                Some("obs_credential_missing"),
            ),
            Err(obs::ObsError::AuthenticationRequired) => Receipt::new_obs(
                action,
                decision,
                ReceiptStatus::Failed,
                None,
                Some("obs_authentication_required"),
            ),
            Err(obs::ObsError::ConnectionFailed | obs::ObsError::HandshakeFailed) => {
                Receipt::new_obs(
                    action,
                    decision,
                    ReceiptStatus::Failed,
                    None,
                    Some("obs_connection_failed"),
                )
            }
            Err(obs::ObsError::ProtocolFailed | obs::ObsError::RequestFailed) => Receipt::new_obs(
                action,
                decision,
                ReceiptStatus::Failed,
                None,
                Some("obs_protocol_or_request_failed"),
            ),
            Err(obs::ObsError::ResponseTooLarge | obs::ObsError::DeadlineExceeded) => {
                Receipt::new_obs(
                    action,
                    decision,
                    ReceiptStatus::Failed,
                    None,
                    Some("obs_response_or_deadline_bound"),
                )
            }
        }
    }
}

fn receipt_without_evidence(
    action: &Action,
    decision: Disposition,
    status: ReceiptStatus,
    error_code: Option<&'static str>,
) -> Result<Receipt, ContractError> {
    if action.kind() == "screen.capture" {
        Receipt::new_desktop(action, decision, status, None, error_code)
    } else if action.kind().starts_with("obs.") {
        Receipt::new_obs(action, decision, status, None, error_code)
    } else {
        Receipt::new(action, decision, status, None, error_code)
    }
}

#[cfg(test)]
mod tests {
    use crate::contracts::{Approval, ProposedAction};
    use crate::receipts::{DESKTOP_RECEIPT_SCHEMA_VERSION, ReceiptStatus};

    use super::*;

    fn action(json: &str) -> Action {
        let proposal: ProposedAction = match serde_json::from_str(json) {
            Ok(value) => value,
            Err(error) => panic!("fixture must parse: {error}"),
        };
        match proposal.normalize() {
            Ok(value) => value,
            Err(error) => panic!("fixture must normalize: {error}"),
        }
    }

    #[test]
    fn approval_must_bind_to_exact_action() {
        let first = action(r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","one"]}}"#);
        let second = action(r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","two"]}}"#);
        let approval = Approval::allow_once(&first, "human");
        let receipt = Runtime::simulated().run(&second, Some(&approval));
        assert!(receipt.is_ok());
        assert_eq!(receipt.ok().map(|r| r.status), Some(ReceiptStatus::ApprovalRequired));
    }

    #[test]
    fn approved_effect_is_only_simulated_by_default() {
        let action = action(r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","hello"]}}"#);
        let approval = Approval::allow_once(&action, "human");
        let receipt = Runtime::simulated().run(&action, Some(&approval));
        assert_eq!(receipt.ok().map(|r| r.status), Some(ReceiptStatus::Simulated));
    }

    #[test]
    fn simulated_screen_capture_uses_desktop_receipt_contract() {
        let action = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"screen.capture"}"#,
        );
        let receipt = Runtime::simulated().run(&action, None);
        assert_eq!(
            receipt.ok().map(|value| value.schema_version),
            Some(DESKTOP_RECEIPT_SCHEMA_VERSION)
        );
    }
}
