use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tungstenite::client::client_with_config;
use tungstenite::protocol::WebSocketConfig;
use tungstenite::{Message, WebSocket};
use zeroize::Zeroizing;

use super::{ObsError, MAX_OBS_MESSAGE_BYTES};

const OBS_RPC_VERSION: u64 = 1;
const OBS_IO_TIMEOUT: Duration = Duration::from_secs(3);

pub(super) trait ObsTransport {
    fn request(
        &mut self,
        request_type: &str,
        request_id: &str,
        request_data: Option<Value>,
    ) -> Result<Value, ObsError>;
}

pub(super) struct LiveObsTransport {
    socket: WebSocket<TcpStream>,
}

impl LiveObsTransport {
    pub(super) fn connect(port: u16, password: Option<&str>) -> Result<Self, ObsError> {
        let address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
        let stream = TcpStream::connect_timeout(&address, OBS_IO_TIMEOUT)
            .map_err(|_| ObsError::ConnectionFailed)?;
        stream
            .set_read_timeout(Some(OBS_IO_TIMEOUT))
            .map_err(|_| ObsError::ConnectionFailed)?;
        stream
            .set_write_timeout(Some(OBS_IO_TIMEOUT))
            .map_err(|_| ObsError::ConnectionFailed)?;

        let endpoint = format!("ws://127.0.0.1:{port}/");
        let websocket_config = WebSocketConfig::default()
            .read_buffer_size(16 * 1024)
            .write_buffer_size(16 * 1024)
            .max_message_size(Some(MAX_OBS_MESSAGE_BYTES))
            .max_frame_size(Some(MAX_OBS_MESSAGE_BYTES));
        let (mut socket, _) = client_with_config(endpoint.as_str(), stream, Some(websocket_config))
            .map_err(|_| ObsError::HandshakeFailed)?;

        let hello = read_json_message(&mut socket)?;
        if hello.get("op").and_then(Value::as_u64) != Some(0) {
            return Err(ObsError::ProtocolFailed);
        }
        let hello_data = hello.get("d").ok_or(ObsError::ProtocolFailed)?;
        let server_rpc = hello_data
            .get("rpcVersion")
            .and_then(Value::as_u64)
            .ok_or(ObsError::ProtocolFailed)?;
        if server_rpc < OBS_RPC_VERSION {
            return Err(ObsError::ProtocolFailed);
        }

        let authentication = match hello_data.get("authentication") {
            Some(authentication) => {
                let password = password.ok_or(ObsError::AuthenticationRequired)?;
                let challenge = authentication
                    .get("challenge")
                    .and_then(Value::as_str)
                    .ok_or(ObsError::ProtocolFailed)?;
                let salt = authentication
                    .get("salt")
                    .and_then(Value::as_str)
                    .ok_or(ObsError::ProtocolFailed)?;
                Some(authentication_response(password, salt, challenge))
            }
            None => None,
        };

        let identify = IdentifyMessage {
            op: 1,
            d: IdentifyData {
                rpc_version: OBS_RPC_VERSION,
                authentication: authentication.as_ref().map(|value| value.as_str()),
                event_subscriptions: 0,
            },
        };
        let identify_payload = Zeroizing::new(
            serde_json::to_string(&identify).map_err(|_| ObsError::ProtocolFailed)?,
        );
        socket
            .send(Message::text(identify_payload.as_str()))
            .map_err(|_| ObsError::ProtocolFailed)?;

        let identified = read_json_message(&mut socket)?;
        if identified.get("op").and_then(Value::as_u64) != Some(2) {
            return Err(ObsError::ProtocolFailed);
        }
        let negotiated_rpc = identified
            .get("d")
            .and_then(|value| value.get("negotiatedRpcVersion"))
            .and_then(Value::as_u64)
            .ok_or(ObsError::ProtocolFailed)?;
        if negotiated_rpc != OBS_RPC_VERSION {
            return Err(ObsError::ProtocolFailed);
        }

        Ok(Self { socket })
    }
}

impl ObsTransport for LiveObsTransport {
    fn request(
        &mut self,
        request_type: &str,
        request_id: &str,
        request_data: Option<Value>,
    ) -> Result<Value, ObsError> {
        let mut data = serde_json::Map::new();
        data.insert(
            "requestType".to_owned(),
            Value::String(request_type.to_owned()),
        );
        data.insert(
            "requestId".to_owned(),
            Value::String(request_id.to_owned()),
        );
        if let Some(request_data) = request_data {
            data.insert("requestData".to_owned(), request_data);
        }
        let payload = json!({"op": 6, "d": Value::Object(data)});
        let payload = serde_json::to_string(&payload).map_err(|_| ObsError::ProtocolFailed)?;
        self.socket
            .send(Message::text(payload))
            .map_err(|_| ObsError::ProtocolFailed)?;

        loop {
            let response = read_json_message(&mut self.socket)?;
            match response.get("op").and_then(Value::as_u64) {
                Some(5) => continue,
                Some(7) => {
                    let data = response.get("d").ok_or(ObsError::ProtocolFailed)?;
                    if data.get("requestId").and_then(Value::as_str) != Some(request_id)
                        || data.get("requestType").and_then(Value::as_str) != Some(request_type)
                    {
                        return Err(ObsError::ProtocolFailed);
                    }
                    let status = data
                        .get("requestStatus")
                        .ok_or(ObsError::ProtocolFailed)?;
                    if status.get("result").and_then(Value::as_bool) != Some(true) {
                        return Err(ObsError::RequestFailed);
                    }
                    return Ok(response);
                }
                _ => return Err(ObsError::ProtocolFailed),
            }
        }
    }
}

#[derive(Serialize)]
struct IdentifyMessage<'a> {
    op: u8,
    d: IdentifyData<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IdentifyData<'a> {
    rpc_version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    authentication: Option<&'a str>,
    event_subscriptions: u64,
}

fn read_json_message(socket: &mut WebSocket<TcpStream>) -> Result<Value, ObsError> {
    loop {
        let message = socket.read().map_err(|_| ObsError::ProtocolFailed)?;
        match message {
            Message::Text(text) => {
                if text.len() > MAX_OBS_MESSAGE_BYTES {
                    return Err(ObsError::ResponseTooLarge);
                }
                return serde_json::from_str(text.as_str()).map_err(|_| ObsError::ProtocolFailed);
            }
            Message::Ping(_) | Message::Pong(_) => continue,
            Message::Close(_) => return Err(ObsError::ProtocolFailed),
            Message::Binary(_) | Message::Frame(_) => return Err(ObsError::ProtocolFailed),
        }
    }
}

fn authentication_response(password: &str, salt: &str, challenge: &str) -> Zeroizing<String> {
    let mut secret_material = Zeroizing::new(String::with_capacity(password.len() + salt.len()));
    secret_material.push_str(password);
    secret_material.push_str(salt);
    let secret_hash = Zeroizing::new(Sha256::digest(secret_material.as_bytes()).to_vec());
    let base64_secret = Zeroizing::new(STANDARD.encode(secret_hash.as_slice()));

    let mut challenge_material =
        Zeroizing::new(String::with_capacity(base64_secret.len() + challenge.len()));
    challenge_material.push_str(base64_secret.as_str());
    challenge_material.push_str(challenge);
    let auth_hash = Zeroizing::new(Sha256::digest(challenge_material.as_bytes()).to_vec());
    Zeroizing::new(STANDARD.encode(auth_hash.as_slice()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obs_authentication_matches_protocol_vector() {
        let response = authentication_response(
            "supersecretpassword",
            "lM1GncleQOaCu9lT1yeUZhFYnqhsLLP1G5lAGo3ixaI=",
            "+IxH4CnCiqpX1rM9scsNynZzbOe4KhDeYcTNS3PDaeY=",
        );
        assert_eq!(
            response.as_str(),
            "Dj6cLS+jrNA0HpCArRg0Z/Fc+YHdt2FQfAvgD1mip6Y="
        );
    }
}
