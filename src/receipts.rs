use serde::Serialize;

use crate::contracts::{canonical_hash, Action, ContractError, RECEIPT_SCHEMA_VERSION};
use crate::policy::Disposition;

pub const OBS_RECEIPT_SCHEMA_VERSION: &str = "qsol-chatgpt-receipt/3";
pub const DESKTOP_RECEIPT_SCHEMA_VERSION: &str = "qsol-chatgpt-receipt/4";
pub const SCREENCAST_RECEIPT_SCHEMA_VERSION: &str = "qsol-chatgpt-receipt/5";

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
pub struct ScreencastEvidence {
    pub backend: String,
    pub frame_chain_sha256: String,
    pub frames_observed: u32,
    pub payload_bytes_hashed: u64,
    pub duration_ms: u64,
    pub width: u32,
    pub height: u32,
    pub framerate_num: u32,
    pub framerate_denom: u32,
    pub source_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portal_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portal_height: Option<u32>,
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
    RecordStatus { active: bool, paused: bool },
    StreamStatus { active: bool, reconnecting: bool },
    Mutation { acknowledged: bool },
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
    pub screencast_evidence: Option<ScreencastEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<&'static str>,
}

#[derive(Serialize)]
struct ReceiptIdentityV2<'a> { schema_version: &'static str, action_id: &'a str, kind: &'a str, decision: Disposition, status: ReceiptStatus, evidence: &'a Option<ExecutionEvidence>, error_code: Option<&'static str> }
#[derive(Serialize)]
struct ReceiptIdentityV3<'a> { schema_version: &'static str, action_id: &'a str, kind: &'a str, decision: Disposition, status: ReceiptStatus, obs_evidence: &'a Option<ObsEvidence>, error_code: Option<&'static str> }
#[derive(Serialize)]
struct ReceiptIdentityV4<'a> { schema_version: &'static str, action_id: &'a str, kind: &'a str, decision: Disposition, status: ReceiptStatus, desktop_evidence: &'a Option<DesktopEvidence>, error_code: Option<&'static str> }
#[derive(Serialize)]
struct ReceiptIdentityV5<'a> { schema_version: &'static str, action_id: &'a str, kind: &'a str, decision: Disposition, status: ReceiptStatus, screencast_evidence: &'a Option<ScreencastEvidence>, error_code: Option<&'static str> }

impl Receipt {
    pub fn new(action: &Action, decision: Disposition, status: ReceiptStatus, evidence: Option<ExecutionEvidence>, error_code: Option<&'static str>) -> Result<Self, ContractError> {
        let identity = ReceiptIdentityV2 { schema_version: RECEIPT_SCHEMA_VERSION, action_id: action.id(), kind: action.kind(), decision, status, evidence: &evidence, error_code };
        let receipt_id = canonical_hash(&identity)?;
        Ok(Self { schema_version: RECEIPT_SCHEMA_VERSION, receipt_id, action_id: action.id().to_owned(), kind: action.kind().to_owned(), decision, status, evidence, obs_evidence: None, desktop_evidence: None, screencast_evidence: None, error_code })
    }

    pub fn new_obs(action: &Action, decision: Disposition, status: ReceiptStatus, obs_evidence: Option<ObsEvidence>, error_code: Option<&'static str>) -> Result<Self, ContractError> {
        let identity = ReceiptIdentityV3 { schema_version: OBS_RECEIPT_SCHEMA_VERSION, action_id: action.id(), kind: action.kind(), decision, status, obs_evidence: &obs_evidence, error_code };
        let receipt_id = canonical_hash(&identity)?;
        Ok(Self { schema_version: OBS_RECEIPT_SCHEMA_VERSION, receipt_id, action_id: action.id().to_owned(), kind: action.kind().to_owned(), decision, status, evidence: None, obs_evidence, desktop_evidence: None, screencast_evidence: None, error_code })
    }

    pub(crate) fn new_desktop(action: &Action, decision: Disposition, status: ReceiptStatus, desktop_evidence: Option<DesktopEvidence>, error_code: Option<&'static str>) -> Result<Self, ContractError> {
        let identity = ReceiptIdentityV4 { schema_version: DESKTOP_RECEIPT_SCHEMA_VERSION, action_id: action.id(), kind: action.kind(), decision, status, desktop_evidence: &desktop_evidence, error_code };
        let receipt_id = canonical_hash(&identity)?;
        Ok(Self { schema_version: DESKTOP_RECEIPT_SCHEMA_VERSION, receipt_id, action_id: action.id().to_owned(), kind: action.kind().to_owned(), decision, status, evidence: None, obs_evidence: None, desktop_evidence, screencast_evidence: None, error_code })
    }

    pub(crate) fn new_screencast(action: &Action, decision: Disposition, status: ReceiptStatus, screencast_evidence: Option<ScreencastEvidence>, error_code: Option<&'static str>) -> Result<Self, ContractError> {
        let identity = ReceiptIdentityV5 { schema_version: SCREENCAST_RECEIPT_SCHEMA_VERSION, action_id: action.id(), kind: action.kind(), decision, status, screencast_evidence: &screencast_evidence, error_code };
        let receipt_id = canonical_hash(&identity)?;
        Ok(Self { schema_version: SCREENCAST_RECEIPT_SCHEMA_VERSION, receipt_id, action_id: action.id().to_owned(), kind: action.kind().to_owned(), decision, status, evidence: None, obs_evidence: None, desktop_evidence: None, screencast_evidence, error_code })
    }
}

#[cfg(test)]
mod tests {
    use crate::contracts::{ProposedAction, PROPOSAL_SCHEMA_VERSION};
    use super::*;

    fn action(kind: &str, args: serde_json::Map<String, serde_json::Value>) -> Action {
        let proposal = ProposedAction { schema_version: PROPOSAL_SCHEMA_VERSION.to_owned(), kind: kind.to_owned(), args: args.into_iter().collect(), requested_by: "agent".to_owned(), credential_handles: Vec::new() };
        match proposal.normalize() { Ok(value) => value, Err(error) => panic!("action fixture failed: {error}") }
    }

    #[test]
    fn screencast_constructor_serializes_only_v5_evidence() {
        let mut args = serde_json::Map::new();
        args.insert("max_frames".to_owned(), serde_json::json!(60));
        args.insert("max_duration_ms".to_owned(), serde_json::json!(5000));
        let action = action("screen.observe", args);
        let evidence = ScreencastEvidence { backend: "xdg_screencast_pipewire".to_owned(), frame_chain_sha256: "a".repeat(64), frames_observed: 2, payload_bytes_hashed: 128, duration_ms: 500, width: 1920, height: 1080, framerate_num: 30, framerate_denom: 1, source_kind: "monitor".to_owned(), position_x: Some(0), position_y: Some(0), portal_width: Some(1920), portal_height: Some(1080) };
        let receipt = Receipt::new_screencast(&action, Disposition::ApprovalRequired, ReceiptStatus::Completed, Some(evidence), None);
        let value = match receipt.and_then(|receipt| serde_json::to_value(receipt).map_err(|_| ContractError::Serialization)) { Ok(value) => value, Err(error) => panic!("v5 fixture failed: {error}") };
        assert_eq!(value.get("schema_version").and_then(serde_json::Value::as_str), Some(SCREENCAST_RECEIPT_SCHEMA_VERSION));
        assert!(value.get("screencast_evidence").is_some());
        assert!(value.get("desktop_evidence").is_none());
        assert!(value.get("obs_evidence").is_none());
        assert!(value.get("evidence").is_none());
    }
}
