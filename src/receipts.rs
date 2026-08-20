use serde::Serialize;

use crate::contracts::{canonical_hash, Action, ContractError, RECEIPT_SCHEMA_VERSION};
use crate::executor::ExecutionEvidence;
use crate::policy::Disposition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptStatus {
    Denied,
    ApprovalRequired,
    Simulated,
    Completed,
    Failed,
    Unsupported,
}

#[derive(Debug, Clone, Serialize)]
pub struct Receipt {
    pub schema_version: &'static str,
    pub receipt_id: String,
    pub action_id: String,
    pub kind: String,
    pub decision: Disposition,
    pub status: ReceiptStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<ExecutionEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<&'static str>,
}

#[derive(Serialize)]
struct ReceiptIdentity<'a> {
    schema_version: &'static str,
    action_id: &'a str,
    kind: &'a str,
    decision: Disposition,
    status: ReceiptStatus,
    evidence: &'a Option<ExecutionEvidence>,
    error_code: Option<&'static str>,
}

impl Receipt {
    pub fn new(
        action: &Action,
        decision: Disposition,
        status: ReceiptStatus,
        evidence: Option<ExecutionEvidence>,
        error_code: Option<&'static str>,
    ) -> Result<Self, ContractError> {
        let identity = ReceiptIdentity {
            schema_version: RECEIPT_SCHEMA_VERSION,
            action_id: action.id(),
            kind: action.kind(),
            decision,
            status,
            evidence: &evidence,
            error_code,
        };
        let receipt_id = canonical_hash(&identity)?;
        Ok(Self {
            schema_version: RECEIPT_SCHEMA_VERSION,
            receipt_id,
            action_id: action.id().to_owned(),
            kind: action.kind().to_owned(),
            decision,
            status,
            evidence,
            error_code,
        })
    }
}
