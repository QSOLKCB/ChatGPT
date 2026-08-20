use serde::Serialize;

use crate::contracts::{canonical_hash, Action, ContractError, RECEIPT_SCHEMA_VERSION};
use crate::policy::Disposition;

pub const OBS_RECEIPT_SCHEMA_VERSION: &str = "qsol-chatgpt-receipt/3";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExecutionEvidence {
    pub exit_code: Option<i32>,
    pub stdout_sha256: String,
    pub stdout_bytes: usize,
    pub stderr_sha256: String,
    pub stderr_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObsEvidence {
    pub request_type: String,
    pub response_sha256: String,
    pub response_bytes: usize,
    pub observation: ObsObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "observation_type", rename_all = "snake_case")]
pub enum ObsObservation {
    Version {
        obs_version_sha256: String,
        obs_version_bytes: usize,
        obs_websocket_version_sha256: String,
        obs_websocket_version_bytes: usize,
        rpc_version: u64,
        available_request_count: usize,
    },
    SceneList {
        current_program_scene_sha256: String,
        current_program_scene_bytes: usize,
        scene_list_sha256: String,
        scene_count: usize,
    },
    CurrentScene {
        scene_name_sha256: String,
        scene_name_bytes: usize,
    },
    RecordStatus {
        active: bool,
        paused: bool,
    },
    StreamStatus {
        active: bool,
        reconnecting: bool,
    },
    Mutation {
        acknowledged: bool,
    },
}

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
    pub obs_evidence: Option<ObsEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<&'static str>,
}

#[derive(Serialize)]
struct ReceiptIdentityV2<'a> {
    schema_version: &'static str,
    action_id: &'a str,
    kind: &'a str,
    decision: Disposition,
    status: ReceiptStatus,
    evidence: &'a Option<ExecutionEvidence>,
    error_code: Option<&'static str>,
}

#[derive(Serialize)]
struct ReceiptIdentityV3<'a> {
    schema_version: &'static str,
    action_id: &'a str,
    kind: &'a str,
    decision: Disposition,
    status: ReceiptStatus,
    obs_evidence: &'a Option<ObsEvidence>,
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
        let identity = ReceiptIdentityV2 {
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
            obs_evidence: None,
            error_code,
        })
    }

    pub fn new_obs(
        action: &Action,
        decision: Disposition,
        status: ReceiptStatus,
        obs_evidence: Option<ObsEvidence>,
        error_code: Option<&'static str>,
    ) -> Result<Self, ContractError> {
        let identity = ReceiptIdentityV3 {
            schema_version: OBS_RECEIPT_SCHEMA_VERSION,
            action_id: action.id(),
            kind: action.kind(),
            decision,
            status,
            obs_evidence: &obs_evidence,
            error_code,
        };
        let receipt_id = canonical_hash(&identity)?;
        Ok(Self {
            schema_version: OBS_RECEIPT_SCHEMA_VERSION,
            receipt_id,
            action_id: action.id().to_owned(),
            kind: action.kind().to_owned(),
            decision,
            status,
            evidence: None,
            obs_evidence,
            error_code,
        })
    }
}
