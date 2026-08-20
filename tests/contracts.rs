use std::fs;
use std::path::Path;

use qsol_chatgpt::contracts::{Approval, ProposedAction};
use qsol_chatgpt::policy::Disposition;
use qsol_chatgpt::receipts::{ObsEvidence, ObsObservation, Receipt, ReceiptStatus};
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
            assert!(properties.contains_key(key), "undeclared property in representative receipt: {key}");
        }
    }
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for key in required.iter().filter_map(Value::as_str) {
            assert!(object.contains_key(key), "missing required representative property: {key}");
        }
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
    let proposal: ProposedAction = match serde_json::from_str(
        r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.scene.current","args":{"obs_port":4455}}"#,
    ) {
        Ok(value) => value,
        Err(error) => panic!("fixture parse failed: {error}"),
    };
    let action = match proposal.normalize() {
        Ok(value) => value,
        Err(error) => panic!("normalize failed: {error}"),
    };
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

    let obs_evidence = match serialized.get("obs_evidence") {
        Some(value) => value,
        None => panic!("representative OBS receipt must contain obs_evidence"),
    };
    let obs_evidence_schema = match v3
        .get("properties")
        .and_then(|value| value.get("obs_evidence"))
    {
        Some(value) => value,
        None => panic!("v3 schema must declare obs_evidence"),
    };
    assert_declared_object_shape(obs_evidence, obs_evidence_schema);

    let observation = match obs_evidence.get("observation") {
        Some(value) => value,
        None => panic!("representative OBS evidence must contain observation"),
    };
    let observation_type = match observation.get("observation_type").and_then(Value::as_str) {
        Some(value) => value,
        None => panic!("representative observation must contain observation_type"),
    };
    let observation_branches = match obs_evidence_schema
        .get("properties")
        .and_then(|value| value.get("observation"))
        .and_then(|value| value.get("oneOf"))
        .and_then(Value::as_array)
    {
        Some(value) => value,
        None => panic!("v3 schema must publish typed observation branches"),
    };
    let matching_branch = observation_branches.iter().find(|branch| {
        branch
            .get("properties")
            .and_then(|value| value.get("observation_type"))
            .and_then(|value| value.get("const"))
            .and_then(Value::as_str)
            == Some(observation_type)
    });
    match matching_branch {
        Some(branch) => assert_declared_object_shape(observation, branch),
        None => panic!("representative observation type is not published by the v3 schema"),
    }
}

#[test]
fn exact_approval_binding_survives_serialization_boundary() {
    let first: ProposedAction = match serde_json::from_str(
        r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","one"]}}"#,
    ) {
        Ok(value) => value,
        Err(error) => panic!("fixture parse failed: {error}"),
    };
    let second: ProposedAction = match serde_json::from_str(
        r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","two"]}}"#,
    ) {
        Ok(value) => value,
        Err(error) => panic!("fixture parse failed: {error}"),
    };
    let first = match first.normalize() {
        Ok(value) => value,
        Err(error) => panic!("normalize failed: {error}"),
    };
    let second = match second.normalize() {
        Ok(value) => value,
        Err(error) => panic!("normalize failed: {error}"),
    };
    let approval = Approval::allow_once(&first, "human");
    let receipt = Runtime::simulated().run(&second, Some(&approval));
    assert_eq!(receipt.ok().map(|r| r.status), Some(ReceiptStatus::ApprovalRequired));
}
