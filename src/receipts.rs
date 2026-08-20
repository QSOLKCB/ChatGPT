use serde::Serialize;

use crate::contracts::{canonical_hash, Action, ContractError, RECEIPT_SCHEMA_VERSION};
use crate::policy::Disposition;

pub const OBS_RECEIPT_SCHEMA_VERSION: &str = "qsol-chatgpt-receipt/3";
pub const DESKTOP_RECEIPT_SCHEMA_VERSION: &str = "qsol-chatgpt-receipt/4";

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
pub struct DesktopEvidence {
    pub backend: String,
    pub image_sha256: String,
    pub image_bytes: usize,
    pub width: u32,
    pub height: u32,
    pub image_format: String,
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
    pub desktop_evidence: Option<DesktopEvidence>,
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

#[derive(Serialize)]
struct ReceiptIdentityV4<'a> {
    schema_version: &'static str,
    action_id: &'a str,
    kind: &'a str,
    decision: Disposition,
    status: ReceiptStatus,
    desktop_evidence: &'a Option<DesktopEvidence>,
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
            desktop_evidence: None,
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
            desktop_evidence: None,
            error_code,
        })
    }

    pub(crate) fn new_desktop(
        action: &Action,
        decision: Disposition,
        status: ReceiptStatus,
        desktop_evidence: Option<DesktopEvidence>,
        error_code: Option<&'static str>,
    ) -> Result<Self, ContractError> {
        let identity = ReceiptIdentityV4 {
            schema_version: DESKTOP_RECEIPT_SCHEMA_VERSION,
            action_id: action.id(),
            kind: action.kind(),
            decision,
            status,
            desktop_evidence: &desktop_evidence,
            error_code,
        };
        let receipt_id = canonical_hash(&identity)?;
        Ok(Self {
            schema_version: DESKTOP_RECEIPT_SCHEMA_VERSION,
            receipt_id,
            action_id: action.id().to_owned(),
            kind: action.kind().to_owned(),
            decision,
            status,
            evidence: None,
            obs_evidence: None,
            desktop_evidence,
            error_code,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::contracts::{ProposedAction, PROPOSAL_SCHEMA_VERSION};

    use super::*;

    fn screen_action() -> Action {
        let proposal = ProposedAction {
            schema_version: PROPOSAL_SCHEMA_VERSION.to_owned(),
            kind: "screen.capture".to_owned(),
            args: Default::default(),
            requested_by: "agent".to_owned(),
            credential_handles: Vec::new(),
        };
        match proposal.normalize() {
            Ok(value) => value,
            Err(error) => panic!("screen action fixture failed: {error}"),
        }
    }

    #[test]
    fn desktop_constructor_serializes_only_the_v4_shape() {
        let action = screen_action();
        let evidence = DesktopEvidence {
            backend: "xdg_desktop_portal_screenshot".to_owned(),
            image_sha256: "a".repeat(64),
            image_bytes: 68,
            width: 1,
            height: 1,
            image_format: "png".to_owned(),
        };
        let receipt = Receipt::new_desktop(
            &action,
            Disposition::Allow,
            ReceiptStatus::Completed,
            Some(evidence),
            None,
        );
        let value = match receipt.and_then(|receipt| {
            serde_json::to_value(receipt).map_err(|_| ContractError::Serialization)
        }) {
            Ok(value) => value,
            Err(error) => panic!("desktop receipt fixture failed: {error}"),
        };
        assert_eq!(
            value.get("schema_version").and_then(serde_json::Value::as_str),
            Some(DESKTOP_RECEIPT_SCHEMA_VERSION)
        );
        assert!(value.get("desktop_evidence").is_some());
        assert!(value.get("obs_evidence").is_none());
        assert!(value.get("evidence").is_none());
    }
}
