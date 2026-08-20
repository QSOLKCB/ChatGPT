use crate::contracts::{Action, Approval, ContractError};
use crate::executor;
use crate::policy::{self, Disposition};
use crate::receipts::{Receipt, ReceiptStatus};

pub struct Runtime {
    execute_effects: bool,
}

impl Runtime {
    pub fn simulated() -> Self {
        Self {
            execute_effects: false,
        }
    }

    pub fn effectful() -> Self {
        Self {
            execute_effects: true,
        }
    }

    pub fn run(&self, action: &Action, approval: Option<&Approval>) -> Result<Receipt, ContractError> {
        let decision = policy::evaluate(action);
        match decision.disposition {
            Disposition::Deny => Receipt::new(
                action,
                decision.disposition,
                ReceiptStatus::Denied,
                None,
                Some(decision.code),
            ),
            Disposition::ApprovalRequired => {
                if !approval.is_some_and(|record| record.permits(action)) {
                    return Receipt::new(
                        action,
                        decision.disposition,
                        ReceiptStatus::ApprovalRequired,
                        None,
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
            return Receipt::new(action, decision, ReceiptStatus::Simulated, None, None);
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
}

#[cfg(test)]
mod tests {
    use crate::contracts::{Approval, ProposedAction};
    use crate::receipts::ReceiptStatus;

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
}
