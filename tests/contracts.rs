use std::fs;
use std::path::Path;

use qsol_chatgpt::contracts::{Approval, ProposedAction};
use qsol_chatgpt::policy::Disposition;
use qsol_chatgpt::receipts::{
    DesktopEvidence, ObsEvidence, ObsObservation, Receipt, ReceiptStatus,
};
use qsol_chatgpt::runtime::Runtime;
use serde_json::Value;

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
            assert!(
                properties.contains_key(key),
                "undeclared property in representative receipt: {key}"
            );
        }
    }
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for key in required.iter().filter_map(Value::as_str) {
            assert!(
                object.contains_key(key),
                "missing required representative property: {key}"
            );
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
    for name in [
        "proposal.schema.json",
        "action.schema.json",
        "approval.schema.json",
        "receipt.schema.json",
    ] {
        let _ = read_schema(name);
    }
}

#[test]
fn published_receipt_schema_accepts_representative_obs_v3_shape() {
    let action = action(
        r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.scene.current","args":{"obs_port":4455}}"#,
    );
    let evidence = ObsEvidence {
        request_type: "GetCurrentProgramScene".to_owned(),
        response_sha256: "a".repeat(64),
        response_bytes: 128,
        observation: ObsObservation::CurrentScene {
            scene_name_sha256: "b".repeat(64),
            scene_name_bytes: 7,
        },
    };
    let receipt = match Receipt::new_obs(
        &action,
        Disposition::Allow,
        ReceiptStatus::Completed,
        Some(evidence),
        None,
    ) {
        Ok(value) => value,
        Err(error) => panic!("receipt construction failed: {error}"),
    };
    let serialized = match serde_json::to_value(&receipt) {
        Ok(value) => value,
        Err(error) => panic!("receipt serialization failed: {error}"),
    };

    let schema = read_schema("receipt.schema.json");
    let v3 = match schema
        .get("$defs")
        .and_then(|value| value.get("obs_receipt_v3"))
    {
        Some(value) => value,
        None => panic!("receipt schema must publish obs_receipt_v3"),
    };
    assert_eq!(
        serialized.get("schema_version").and_then(Value::as_str),
        Some("qsol-chatgpt-receipt/3")
    );
    assert_declared_object_shape(&serialized, v3);
}

#[test]
fn published_receipt_schema_accepts_representative_desktop_v4_shape() {
    let action = action(
        r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"screen.capture"}"#,
    );
    let evidence = DesktopEvidence {
        backend: "xdg_desktop_portal_screenshot".to_owned(),
        image_sha256: "c".repeat(64),
        image_bytes: 4096,
        width: 1920,
        height: 1080,
        image_format: "png".to_owned(),
    };
    let receipt = match Receipt::new_desktop(
        &action,
        Disposition::Allow,
        ReceiptStatus::Completed,
        Some(evidence),
        None,
    ) {
        Ok(value) => value,
        Err(error) => panic!("desktop receipt construction failed: {error}"),
    };
    let serialized = match serde_json::to_value(&receipt) {
        Ok(value) => value,
        Err(error) => panic!("desktop receipt serialization failed: {error}"),
    };

    let schema = read_schema("receipt.schema.json");
    let v4 = match schema
        .get("$defs")
        .and_then(|value| value.get("desktop_receipt_v4"))
    {
        Some(value) => value,
        None => panic!("receipt schema must publish desktop_receipt_v4"),
    };
    assert_eq!(
        serialized.get("schema_version").and_then(Value::as_str),
        Some("qsol-chatgpt-receipt/4")
    );
    assert_declared_object_shape(&serialized, v4);

    let evidence = match serialized.get("desktop_evidence") {
        Some(value) => value,
        None => panic!("desktop receipt must contain desktop_evidence"),
    };
    let evidence_schema = match v4
        .get("properties")
        .and_then(|value| value.get("desktop_evidence"))
    {
        Some(value) => value,
        None => panic!("v4 schema must declare desktop_evidence"),
    };
    assert_declared_object_shape(evidence, evidence_schema);
    let rendered = serialized.to_string();
    assert!(!rendered.contains("file://"));
    assert!(!rendered.contains("PNG"));
}

#[test]
fn exact_approval_binding_survives_serialization_boundary() {
    let first = action(
        r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","one"]}}"#,
    );
    let second = action(
        r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","two"]}}"#,
    );
    let approval = Approval::allow_once(&first, "human");
    let receipt = Runtime::simulated().run(&second, Some(&approval));
    assert_eq!(
        receipt.ok().map(|r| r.status),
        Some(ReceiptStatus::ApprovalRequired)
    );
}
