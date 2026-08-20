use std::collections::BTreeSet;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::contracts::{canonical_hash, Action};
use crate::receipts::{ObsEvidence, ObsObservation};

use super::transport::ObsTransport;
use super::{ObsError, MAX_OBS_MESSAGE_BYTES, MAX_OBS_STRING_CHARS};

const MAX_OBS_SCENES: usize = 256;
const MAX_AVAILABLE_REQUESTS: usize = 1024;

pub(super) fn execute_with_transport(
    action: &Action,
    transport: &mut dyn ObsTransport,
) -> Result<ObsEvidence, ObsError> {
    let (request_type, request_data) = map_action(action)?;

    let version_request_id = format!("{}:version", action.id());
    let version_response = transport.request("GetVersion", &version_request_id, None)?;
    let available = available_requests(&version_response)?;

    if request_type == "GetVersion" {
        return obs_evidence(
            "GetVersion",
            &version_response,
            version_observation(&version_response)?,
        );
    }

    if !available.contains(request_type) {
        return Err(ObsError::UnsupportedRequest);
    }

    let request_id = format!("{}:action", action.id());
    let response = transport.request(request_type, &request_id, request_data)?;
    let observation = observation_for(action, &response)?;
    obs_evidence(request_type, &response, observation)
}

fn map_action(action: &Action) -> Result<(&'static str, Option<Value>), ObsError> {
    match action.kind() {
        "obs.version.get" => Ok(("GetVersion", None)),
        "obs.scene.list" => Ok(("GetSceneList", None)),
        "obs.scene.current" => Ok(("GetCurrentProgramScene", None)),
        "obs.record.status" => Ok(("GetRecordStatus", None)),
        "obs.stream.status" => Ok(("GetStreamStatus", None)),
        "obs.scene.set" => {
            let scene_name = action
                .args()
                .get("scene_name")
                .and_then(Value::as_str)
                .ok_or(ObsError::InvalidArguments)?;
            if scene_name.trim().is_empty()
                || scene_name.chars().count() > MAX_OBS_STRING_CHARS
                || scene_name.contains('\0')
            {
                return Err(ObsError::InvalidArguments);
            }
            Ok((
                "SetCurrentProgramScene",
                Some(json!({"sceneName": scene_name})),
            ))
        }
        "obs.record.start" => Ok(("StartRecord", None)),
        "obs.record.stop" => Ok(("StopRecord", None)),
        "obs.stream.stop" => Ok(("StopStream", None)),
        _ => Err(ObsError::UnsupportedAction),
    }
}

fn observation_for(action: &Action, response: &Value) -> Result<ObsObservation, ObsError> {
    match action.kind() {
        "obs.scene.list" => scene_list_observation(response),
        "obs.scene.current" => {
            let data = response_data(response)?;
            let scene_name = bounded_str(data, "currentProgramSceneName")?;
            let (scene_name_sha256, scene_name_bytes) = sensitive_string_fingerprint(scene_name);
            Ok(ObsObservation::CurrentScene {
                scene_name_sha256,
                scene_name_bytes,
            })
        }
        "obs.record.status" => {
            let data = response_data(response)?;
            let active = required_bool(data, "outputActive")?;
            let paused = required_bool(data, "outputPaused")?;
            Ok(ObsObservation::RecordStatus { active, paused })
        }
        "obs.stream.status" => {
            let data = response_data(response)?;
            let active = required_bool(data, "outputActive")?;
            let reconnecting = required_bool(data, "outputReconnecting")?;
            Ok(ObsObservation::StreamStatus {
                active,
                reconnecting,
            })
        }
        "obs.scene.set" | "obs.record.start" | "obs.record.stop" | "obs.stream.stop" => {
            Ok(ObsObservation::Mutation { acknowledged: true })
        }
        _ => Err(ObsError::UnsupportedAction),
    }
}

fn version_observation(response: &Value) -> Result<ObsObservation, ObsError> {
    let data = response_data(response)?;
    let obs_version = bounded_str(data, "obsVersion")?;
    let obs_websocket_version = bounded_str(data, "obsWebSocketVersion")?;
    let (obs_version_sha256, obs_version_bytes) = sensitive_string_fingerprint(obs_version);
    let (obs_websocket_version_sha256, obs_websocket_version_bytes) =
        sensitive_string_fingerprint(obs_websocket_version);
    let rpc_version = data
        .get("rpcVersion")
        .and_then(Value::as_u64)
        .ok_or(ObsError::ProtocolFailed)?;
    let available_request_count = available_requests(response)?.len();
    Ok(ObsObservation::Version {
        obs_version_sha256,
        obs_version_bytes,
        obs_websocket_version_sha256,
        obs_websocket_version_bytes,
        rpc_version,
        available_request_count,
    })
}

fn scene_list_observation(response: &Value) -> Result<ObsObservation, ObsError> {
    let data = response_data(response)?;
    let current_program_scene = bounded_str(data, "currentProgramSceneName")?;
    let raw_scenes = data
        .get("scenes")
        .and_then(Value::as_array)
        .ok_or(ObsError::ProtocolFailed)?;
    if raw_scenes.len() > MAX_OBS_SCENES {
        return Err(ObsError::ResponseTooLarge);
    }

    for raw_scene in raw_scenes {
        let _ = bounded_str(raw_scene, "sceneName")?;
    }

    let (current_program_scene_sha256, current_program_scene_bytes) =
        sensitive_string_fingerprint(current_program_scene);
    let scene_list_sha256 = canonical_hash(raw_scenes).map_err(|_| ObsError::ProtocolFailed)?;
    Ok(ObsObservation::SceneList {
        current_program_scene_sha256,
        current_program_scene_bytes,
        scene_list_sha256,
        scene_count: raw_scenes.len(),
    })
}

fn available_requests(response: &Value) -> Result<BTreeSet<String>, ObsError> {
    let data = response_data(response)?;
    let raw_requests = data
        .get("availableRequests")
        .and_then(Value::as_array)
        .ok_or(ObsError::ProtocolFailed)?;
    if raw_requests.len() > MAX_AVAILABLE_REQUESTS {
        return Err(ObsError::ResponseTooLarge);
    }

    let mut requests = BTreeSet::new();
    for raw in raw_requests {
        let request = raw.as_str().ok_or(ObsError::ProtocolFailed)?;
        if request.chars().count() > 128 {
            return Err(ObsError::ResponseTooLarge);
        }
        requests.insert(request.to_owned());
    }
    Ok(requests)
}

fn response_data(response: &Value) -> Result<&Value, ObsError> {
    response
        .get("d")
        .and_then(|value| value.get("responseData"))
        .ok_or(ObsError::ProtocolFailed)
}

fn bounded_str<'a>(parent: &'a Value, key: &str) -> Result<&'a str, ObsError> {
    let value = parent
        .get(key)
        .and_then(Value::as_str)
        .ok_or(ObsError::ProtocolFailed)?;
    if value.chars().count() > MAX_OBS_STRING_CHARS {
        return Err(ObsError::ResponseTooLarge);
    }
    Ok(value)
}

fn sensitive_string_fingerprint(value: &str) -> (String, usize) {
    (
        format!("{:x}", Sha256::digest(value.as_bytes())),
        value.len(),
    )
}

fn required_bool(parent: &Value, key: &str) -> Result<bool, ObsError> {
    parent
        .get(key)
        .and_then(Value::as_bool)
        .ok_or(ObsError::ProtocolFailed)
}

fn obs_evidence(
    request_type: &str,
    response: &Value,
    observation: ObsObservation,
) -> Result<ObsEvidence, ObsError> {
    let response_sha256 = canonical_hash(response).map_err(|_| ObsError::ProtocolFailed)?;
    let response_bytes = serde_json::to_vec(response)
        .map_err(|_| ObsError::ProtocolFailed)?
        .len();
    if response_bytes > MAX_OBS_MESSAGE_BYTES {
        return Err(ObsError::ResponseTooLarge);
    }
    Ok(ObsEvidence {
        request_type: request_type.to_owned(),
        response_sha256,
        response_bytes,
        observation,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::contracts::{ProposedAction, PROPOSAL_SCHEMA_VERSION};

    use super::*;

    struct FakeTransport {
        calls: Vec<String>,
        responses: BTreeMap<String, Value>,
    }

    impl ObsTransport for FakeTransport {
        fn request(
            &mut self,
            request_type: &str,
            request_id: &str,
            _request_data: Option<Value>,
        ) -> Result<Value, ObsError> {
            self.calls.push(request_type.to_owned());
            let Some(response_data) = self.responses.get(request_type).cloned() else {
                return Err(ObsError::UnsupportedRequest);
            };
            Ok(json!({
                "op": 7,
                "d": {
                    "requestType": request_type,
                    "requestId": request_id,
                    "requestStatus": {"result": true, "code": 100},
                    "responseData": response_data
                }
            }))
        }
    }

    fn action(kind: &str, mut args: BTreeMap<String, Value>) -> Action {
        args.insert("obs_port".to_owned(), json!(4455));
        let proposal = ProposedAction {
            schema_version: PROPOSAL_SCHEMA_VERSION.to_owned(),
            kind: kind.to_owned(),
            args,
            requested_by: "agent".to_owned(),
            credential_handles: Vec::new(),
        };
        match proposal.normalize() {
            Ok(value) => value,
            Err(error) => panic!("fixture normalization failed: {error}"),
        }
    }

    fn fake_transport() -> FakeTransport {
        let mut responses = BTreeMap::new();
        responses.insert(
            "GetVersion".to_owned(),
            json!({
                "obsVersion": "32.0.0",
                "obsWebSocketVersion": "5.6.0",
                "rpcVersion": 1,
                "availableRequests": [
                    "GetVersion",
                    "GetSceneList",
                    "GetCurrentProgramScene",
                    "GetRecordStatus",
                    "GetStreamStatus",
                    "SetCurrentProgramScene",
                    "StartRecord",
                    "StopRecord",
                    "StopStream"
                ]
            }),
        );
        responses.insert(
            "GetSceneList".to_owned(),
            json!({
                "currentProgramSceneName": "Desktop",
                "scenes": [
                    {"sceneName": "Desktop"},
                    {"sceneName": "client_secret=do-not-log"}
                ]
            }),
        );
        responses.insert("StartRecord".to_owned(), json!({}));
        FakeTransport {
            calls: Vec::new(),
            responses,
        }
    }

    #[test]
    fn scene_list_uses_hashed_audit_observation() {
        let mut transport = fake_transport();
        let action = action("obs.scene.list", BTreeMap::new());
        let evidence = execute_with_transport(&action, &mut transport);
        assert!(evidence.is_ok());
        assert_eq!(transport.calls, vec!["GetVersion", "GetSceneList"]);
        match evidence {
            Ok(evidence) => {
                match &evidence.observation {
                    ObsObservation::SceneList { scene_count, .. } => assert_eq!(*scene_count, 2),
                    _ => panic!("expected OBS scene-list evidence"),
                }
                let serialized = match serde_json::to_string(&evidence) {
                    Ok(value) => value,
                    Err(error) => panic!("evidence serialization failed: {error}"),
                };
                assert!(!serialized.contains("Desktop"));
                assert!(!serialized.contains("client_secret=do-not-log"));
            }
            Err(error) => panic!("unexpected OBS error: {error}"),
        }
    }

    #[test]
    fn record_start_maps_without_raw_request_escape() {
        let mut transport = fake_transport();
        let action = action("obs.record.start", BTreeMap::new());
        let evidence = execute_with_transport(&action, &mut transport);
        assert!(evidence.is_ok());
        assert_eq!(transport.calls, vec!["GetVersion", "StartRecord"]);
    }
}
