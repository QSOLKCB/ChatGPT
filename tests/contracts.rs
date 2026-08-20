use std::fs;
use std::path::Path;

use qsol_chatgpt::contracts::{Approval, ProposedAction};
use qsol_chatgpt::runtime::Runtime;
use qsol_chatgpt::receipts::ReceiptStatus;

#[test]
fn machine_schemas_are_valid_json() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for name in [
        "proposal.schema.json",
        "action.schema.json",
        "approval.schema.json",
        "receipt.schema.json",
    ] {
        let path = root.join("schemas").join(name);
        let text = match fs::read_to_string(&path) {
            Ok(value) => value,
            Err(error) => panic!("failed reading {}: {error}", path.display()),
        };
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&text);
        assert!(parsed.is_ok(), "{} must contain valid JSON", path.display());
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
