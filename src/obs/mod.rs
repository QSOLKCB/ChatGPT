mod protocol;
mod transport;

use std::num::NonZeroU16;

use thiserror::Error;

use crate::contracts::{Action, CredentialHandle};
use crate::receipts::ObsEvidence;
use crate::secrets::SecretStore;

use self::protocol::execute_with_transport;
use self::transport::LiveObsTransport;

pub const DEFAULT_OBS_PORT: u16 = 4455;
pub(super) const MAX_OBS_MESSAGE_BYTES: usize = 1024 * 1024;
pub(super) const MAX_OBS_STRING_CHARS: usize = 512;

#[derive(Debug, Clone)]
pub struct ObsConnectionConfig {
    port: NonZeroU16,
    credential_handle: Option<CredentialHandle>,
}

impl ObsConnectionConfig {
    pub fn localhost(
        port: u16,
        credential_handle: Option<CredentialHandle>,
    ) -> Result<Self, ObsConfigError> {
        let port = NonZeroU16::new(port).ok_or(ObsConfigError::InvalidPort)?;
        Ok(Self {
            port,
            credential_handle,
        })
    }

    pub fn port(&self) -> u16 {
        self.port.get()
    }

    pub fn credential_handle(&self) -> Option<&CredentialHandle> {
        self.credential_handle.as_ref()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ObsConfigError {
    #[error("OBS websocket port must be non-zero")]
    InvalidPort,
}

#[derive(Debug, Error)]
pub(crate) enum ObsError {
    #[error("OBS action is not supported")]
    UnsupportedAction,
    #[error("OBS action arguments are invalid")]
    InvalidArguments,
    #[error("OBS credential binding does not match the action")]
    CredentialBindingMismatch,
    #[error("OBS credential is not present in the secret store")]
    MissingCredential,
    #[error("OBS websocket requires authentication")]
    AuthenticationRequired,
    #[error("failed to connect to OBS websocket")]
    ConnectionFailed,
    #[error("OBS websocket handshake failed")]
    HandshakeFailed,
    #[error("OBS websocket protocol exchange failed")]
    ProtocolFailed,
    #[error("OBS websocket request failed")]
    RequestFailed,
    #[error("OBS websocket request is not available on this server")]
    UnsupportedRequest,
    #[error("OBS websocket response exceeded a bounded contract")]
    ResponseTooLarge,
}

pub(crate) fn execute(
    action: &Action,
    config: &ObsConnectionConfig,
    secrets: &SecretStore,
) -> Result<ObsEvidence, ObsError> {
    match config.credential_handle() {
        Some(handle) => {
            if action.credential_handles() != std::slice::from_ref(handle) {
                return Err(ObsError::CredentialBindingMismatch);
            }
            let Some(result) = secrets.with_secret(handle, |password| {
                execute_live(action, config.port(), Some(password))
            }) else {
                return Err(ObsError::MissingCredential);
            };
            result
        }
        None => {
            if !action.credential_handles().is_empty() {
                return Err(ObsError::CredentialBindingMismatch);
            }
            execute_live(action, config.port(), None)
        }
    }
}

fn execute_live(
    action: &Action,
    port: u16,
    password: Option<&str>,
) -> Result<ObsEvidence, ObsError> {
    let mut transport = LiveObsTransport::connect(port, password)?;
    execute_with_transport(action, &mut transport)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_rejects_zero_port() {
        let config = ObsConnectionConfig::localhost(0, None);
        assert!(matches!(config, Err(ObsConfigError::InvalidPort)));
    }
}
