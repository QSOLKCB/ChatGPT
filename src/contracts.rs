use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const PROPOSAL_SCHEMA_VERSION: &str = "qsol-chatgpt-proposal/1";
pub const ACTION_SCHEMA_VERSION: &str = "qsol-chatgpt-action/2";
pub const APPROVAL_SCHEMA_VERSION: &str = "qsol-chatgpt-approval/2";
pub const RECEIPT_SCHEMA_VERSION: &str = "qsol-chatgpt-receipt/2";

const FORBIDDEN_SECRET_KEYS: &[&str] = &[
    "api_key",
    "apikey",
    "authorization",
    "cookie",
    "password",
    "private_key",
    "secret",
    "token",
];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContractError {
    #[error("unsupported proposal schema version")]
    UnsupportedSchema,
    #[error("action kind must not be empty")]
    EmptyKind,
    #[error("requested_by must not be empty")]
    EmptyRequester,
    #[error("raw secret-shaped field is forbidden in action arguments: {0}")]
    RawSecretField(String),
    #[error("invalid credential handle")]
    InvalidCredentialHandle,
    #[error("canonical serialization failed")]
    Serialization,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CredentialHandle(String);

impl CredentialHandle {
    pub fn parse(value: String) -> Result<Self, ContractError> {
        let valid = value.starts_with("cred:")
            && value.len() > 5
            && value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '.' | '_' | '-'));
        if !valid {
            return Err(ContractError::InvalidCredentialHandle);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for CredentialHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CredentialHandle").field(&self.0).finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposedAction {
    pub schema_version: String,
    pub kind: String,
    #[serde(default)]
    pub args: BTreeMap<String, Value>,
    #[serde(default = "default_requester")]
    pub requested_by: String,
    #[serde(default)]
    pub credential_handles: Vec<String>,
}

fn default_requester() -> String {
    "agent".to_owned()
}

#[derive(Debug, Clone, Serialize)]
pub struct Action {
    schema_version: &'static str,
    action_id: String,
    kind: String,
    args: BTreeMap<String, Value>,
    requested_by: String,
    credential_handles: Vec<CredentialHandle>,
}

#[derive(Serialize)]
struct ActionIdentity<'a> {
    schema_version: &'static str,
    kind: &'a str,
    args: &'a BTreeMap<String, Value>,
    requested_by: &'a str,
    credential_handles: &'a [CredentialHandle],
}

impl ProposedAction {
    pub fn normalize(self) -> Result<Action, ContractError> {
        if self.schema_version != PROPOSAL_SCHEMA_VERSION {
            return Err(ContractError::UnsupportedSchema);
        }
        if self.kind.trim().is_empty() {
            return Err(ContractError::EmptyKind);
        }
        if self.requested_by.trim().is_empty() {
            return Err(ContractError::EmptyRequester);
        }
        reject_secret_keys_in_map(&self.args)?;

        let mut credential_handles = self
            .credential_handles
            .into_iter()
            .map(CredentialHandle::parse)
            .collect::<Result<Vec<_>, _>>()?;
        credential_handles.sort();
        credential_handles.dedup();

        let identity = ActionIdentity {
            schema_version: ACTION_SCHEMA_VERSION,
            kind: &self.kind,
            args: &self.args,
            requested_by: &self.requested_by,
            credential_handles: &credential_handles,
        };
        let action_id = canonical_hash(&identity)?;

        Ok(Action {
            schema_version: ACTION_SCHEMA_VERSION,
            action_id,
            kind: self.kind,
            args: self.args,
            requested_by: self.requested_by,
            credential_handles,
        })
    }
}

impl Action {
    pub fn id(&self) -> &str {
        &self.action_id
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn args(&self) -> &BTreeMap<String, Value> {
        &self.args
    }

    pub fn credential_handles(&self) -> &[CredentialHandle] {
        &self.credential_handles
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval {
    pub schema_version: String,
    pub action_id: String,
    pub approved: bool,
    pub approved_by: String,
}

impl Approval {
    pub fn allow_once(action: &Action, approved_by: impl Into<String>) -> Self {
        Self {
            schema_version: APPROVAL_SCHEMA_VERSION.to_owned(),
            action_id: action.id().to_owned(),
            approved: true,
            approved_by: approved_by.into(),
        }
    }

    pub fn permits(&self, action: &Action) -> bool {
        self.schema_version == APPROVAL_SCHEMA_VERSION
            && self.approved
            && !self.approved_by.trim().is_empty()
            && self.action_id == action.id()
    }
}

pub fn canonical_hash<T: Serialize>(value: &T) -> Result<String, ContractError> {
    let bytes = serde_json::to_vec(value).map_err(|_| ContractError::Serialization)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn reject_secret_keys_in_map(map: &BTreeMap<String, Value>) -> Result<(), ContractError> {
    for (key, value) in map {
        let lowered = key.to_ascii_lowercase();
        if FORBIDDEN_SECRET_KEYS.contains(&lowered.as_str()) {
            return Err(ContractError::RawSecretField(key.clone()));
        }
        reject_secret_keys_in_value(value)?;
    }
    Ok(())
}

fn reject_secret_keys_in_value(value: &Value) -> Result<(), ContractError> {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                let lowered = key.to_ascii_lowercase();
                if FORBIDDEN_SECRET_KEYS.contains(&lowered.as_str()) {
                    return Err(ContractError::RawSecretField(key.clone()));
                }
                reject_secret_keys_in_value(nested)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                reject_secret_keys_in_value(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proposal(json: &str) -> ProposedAction {
        match serde_json::from_str(json) {
            Ok(value) => value,
            Err(error) => panic!("fixture must parse: {error}"),
        }
    }

    #[test]
    fn action_identity_is_independent_of_argument_key_order() {
        let left = proposal(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"b":2,"a":1},"requested_by":"agent"}"#,
        )
        .normalize();
        let right = proposal(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"a":1,"b":2},"requested_by":"agent"}"#,
        )
        .normalize();

        assert!(left.is_ok());
        assert!(right.is_ok());
        assert_eq!(left.ok().map(|a| a.action_id), right.ok().map(|a| a.action_id));
    }

    #[test]
    fn raw_secret_fields_are_rejected() {
        let result = proposal(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"api_key":"nope"}}"#,
        )
        .normalize();
        assert!(matches!(result, Err(ContractError::RawSecretField(_))));
    }
}
