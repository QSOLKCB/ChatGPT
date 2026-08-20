use std::fs;
use std::path::Path;

use qsol_chatgpt::contracts::{Approval, ProposedAction};
use qsol_chatgpt::policy::Disposition;
use qsol_chatgpt::receipts::{ObsEvidence, ObsObservation, Receipt, ReceiptStatus};
use qsol_chatgpt::runtime::Runtime;
use serde_json::{json, Value};

fn read_schema(name: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("schemas").join(name);
    let text = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(error) => panic!("failed reading {}: {error}", path.display()),
    };
    match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(error) => panic!("{} must contain valid JSON: {error}", path.display()),
    }
}

fn assert_declared_object_shape(value: &Value, schema: &Value) {
    let object = match value.as_object() {
        Some(value) => value,
        None => panic!("representative value must be an object"),
    };
    let properties = match schema.get("properties").and_then(Value::as_object) {
        Some(value) => value,
        None => panic!("schema branch must publish object properties"),
    };
    if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
        for key in object.keys() {
            assert!(properties.contains_key(key), "undeclared property in representative receipt: {key}");
        }
    }
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for key in required.iter().filter_map(Value::as_str) {
            assert!(object.contains_key(key), "missing required representative property: {key}");
        }
    }
}

fn action(json: &str) -> qsol_chatgpt::contracts::Action {
    let proposal: ProposedAction = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(error) => panic!("fixture parse failed: {error}"),
    };
    match proposal.normalize() {
        Ok(value) => value,
        Err(error) => panic!("normalize failed: {error}"),
    }
}

#[test]
fn machine_schemas_are_valid_json() {
    for name in ["proposal.schema.json", "action.schema.json", "approval.schema.json", "receipt.schema.json"] {
        let _ = read_schema(name);
    }
}

#[test]
fn published_receipt_schema_accepts_representative_obs_v3_shape() {
    let action = action(r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.scene.current","args":{"obs_port":4455}}"#);
    let evidence = ObsEvidence {
        request_type: "GetCurrentProgramScene".to_owned(),
        response_sha256: "a".repeat(64),
        response_bytes: 128,
        observation: ObsObservation::CurrentScene { scene_name_sha256: "b".repeat(64), scene_name_bytes: 7 },
    };
    let receipt = match Receipt::new_obs(&action, Disposition::Allow, ReceiptStatus::Completed, Some(evidence), None) {
        Ok(value) => value,
        Err(error) => panic!("receipt construction failed: {error}"),
    };
    let serialized = match serde_json::to_value(&receipt) {
        Ok(value) => value,
        Err(error) => panic!("receipt serialization failed: {error}"),
    };
    let schema = read_schema("receipt.schema.json");
    let v3 = schema.get("$defs").and_then(|value| value.get("obs_receipt_v3")).unwrap_or_else(|| panic!("receipt schema must publish obs_receipt_v3"));
    assert_declared_object_shape(&serialized, v3);
    let obs_evidence = serialized.get("obs_evidence").unwrap_or_else(|| panic!("representative OBS receipt must contain obs_evidence"));
    let obs_evidence_schema = v3.get("properties").and_then(|value| value.get("obs_evidence")).unwrap_or_else(|| panic!("v3 schema must declare obs_evidence"));
    assert_declared_object_shape(obs_evidence, obs_evidence_schema);
    let observation = obs_evidence.get("observation").unwrap_or_else(|| panic!("representative OBS evidence must contain observation"));
    let observation_type = observation.get("observation_type").and_then(Value::as_str).unwrap_or_else(|| panic!("representative observation must contain observation_type"));
    let observation_branches = obs_evidence_schema.get("properties").and_then(|value| value.get("observation")).and_then(|value| value.get("oneOf")).and_then(Value::as_array).unwrap_or_else(|| panic!("v3 schema must publish typed observation branches"));
    let matching_branch = observation_branches.iter().find(|branch| branch.get("properties").and_then(|value| value.get("observation_type")).and_then(|value| value.get("const")).and_then(Value::as_str) == Some(observation_type));
    match matching_branch { Some(branch) => assert_declared_object_shape(observation, branch), None => panic!("representative observation type is not published by the v3 schema") }
}

#[test]
fn published_receipt_schema_accepts_representative_desktop_v4_shape() {
    let serialized = json!({
        "schema_version": "qsol-chatgpt-receipt/4", "receipt_id": "d".repeat(64), "action_id": "e".repeat(64), "kind": "screen.capture", "decision": "allow", "status": "completed",
        "desktop_evidence": {"backend": "xdg_desktop_portal_screenshot", "image_sha256": "c".repeat(64), "image_bytes": 68, "width": 1, "height": 1, "image_format": "png"}
    });
    let schema = read_schema("receipt.schema.json");
    let v4 = schema.get("$defs").and_then(|value| value.get("desktop_receipt_v4")).unwrap_or_else(|| panic!("receipt schema must publish desktop_receipt_v4"));
    assert_declared_object_shape(&serialized, v4);
    let evidence = serialized.get("desktop_evidence").unwrap_or_else(|| panic!("desktop receipt must contain desktop_evidence"));
    let evidence_schema = v4.get("properties").and_then(|value| value.get("desktop_evidence")).unwrap_or_else(|| panic!("v4 schema must declare desktop_evidence"));
    assert_declared_object_shape(evidence, evidence_schema);
}

#[test]
fn published_receipt_schema_accepts_representative_screencast_v5_shape() {
    let serialized = json!({
        "schema_version": "qsol-chatgpt-receipt/5", "receipt_id": "1".repeat(64), "action_id": "2".repeat(64), "kind": "screen.observe", "decision": "approval_required", "status": "completed",
        "screencast_evidence": {
            "backend": "xdg_screencast_pipewire", "frame_chain_sha256": "3".repeat(64), "frames_observed": 60, "payload_bytes_hashed": 125829120, "duration_ms": 5000,
            "width": 1920, "height": 1080, "framerate_num": 30, "framerate_denom": 1, "source_kind": "monitor", "position_x": 0, "position_y": 0, "portal_width": 1920, "portal_height": 1080
        }
    });
    let schema = read_schema("receipt.schema.json");
    let v5 = schema.get("$defs").and_then(|value| value.get("screencast_receipt_v5")).unwrap_or_else(|| panic!("receipt schema must publish screencast_receipt_v5"));
    assert_declared_object_shape(&serialized, v5);
    let evidence = serialized.get("screencast_evidence").unwrap_or_else(|| panic!("v5 representative must contain screencast_evidence"));
    let evidence_schema = v5.get("properties").and_then(|value| value.get("screencast_evidence")).unwrap_or_else(|| panic!("v5 schema must declare screencast_evidence"));
    assert_declared_object_shape(evidence, evidence_schema);
    let rendered = serialized.to_string();
    assert!(!rendered.contains("restore_token"));
    assert!(!rendered.contains("pipe_wire_node_id"));
    assert!(!rendered.contains("file://"));
}

#[test]
fn exact_approval_binding_survives_serialization_boundary() {
    let first = action(r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","one"]}}"#);
    let second = action(r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","two"]}}"#);
    let approval = Approval::allow_once(&first, "human");
    let receipt = Runtime::simulated().run(&second, Some(&approval));
    assert_eq!(receipt.ok().map(|r| r.status), Some(ReceiptStatus::ApprovalRequired));
}
