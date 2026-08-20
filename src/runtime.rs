use crate::contracts::{Action, Approval, ContractError};
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
            Err(obs::ObsError::ResponseTooLarge) => Receipt::new_obs(
                action,
                decision,
                ReceiptStatus::Failed,
                None,
                Some("obs_response_too_large"),
            ),
        }
    }
}

fn receipt_without_evidence(
    action: &Action,
    decision: Disposition,
    status: ReceiptStatus,
    error_code: Option<&'static str>,
) -> Result<Receipt, ContractError> {
    if action.kind().starts_with("obs.") {
        Receipt::new_obs(action, decision, status, None, error_code)
    } else {
        Receipt::new(action, decision, status, None, error_code)
    }
}

#[cfg(test)]
mod tests {
    use crate::contracts::{Approval, ProposedAction};
    use crate::receipts::{ReceiptStatus, OBS_RECEIPT_SCHEMA_VERSION};

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
    fn live_obs_action_without_config_is_unsupported() {
        let action = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.scene.list"}"#,
        );
        let receipt = Runtime::effectful().run(&action, None);
        match receipt {
            Ok(receipt) => {
                assert_eq!(receipt.status, ReceiptStatus::Unsupported);
                assert_eq!(receipt.schema_version, OBS_RECEIPT_SCHEMA_VERSION);
            }
            Err(error) => panic!("unexpected receipt error: {error}"),
        }
    }

    #[test]
    fn stream_start_cannot_be_overridden_by_approval() {
        let action = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.stream.start"}"#,
        );
        let approval = Approval::allow_once(&action, "human");
        let receipt = Runtime::simulated().run(&action, Some(&approval));
        assert_eq!(receipt.ok().map(|r| r.status), Some(ReceiptStatus::Denied));
    }
}
