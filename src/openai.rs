use crate::contracts::CredentialHandle;
use thiserror::Error;

pub const OPENAI_API_ORIGIN: &str = "https://api.openai.com";
pub const OPENAI_RESPONSES_PATH: &str = "/v1/responses";
pub const OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";

const MAX_MODEL_CHARS: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiConfig {
    credential_handle: CredentialHandle,
    model: String,
}

impl OpenAiConfig {
    pub fn new(
        credential_handle: CredentialHandle,
        model: impl Into<String>,
    ) -> Result<Self, OpenAiConfigError> {
        if !credential_handle.as_str().starts_with("cred:openai.") {
            return Err(OpenAiConfigError::InvalidCredentialHandle);
        }

        let model = model.into();
        let model = model.trim();
        if model.is_empty() {
            return Err(OpenAiConfigError::EmptyModel);
        }
        if model.chars().count() > MAX_MODEL_CHARS {
            return Err(OpenAiConfigError::ModelTooLong);
        }
        if !model
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
        {
            return Err(OpenAiConfigError::InvalidModel);
        }

        Ok(Self {
            credential_handle,
            model: model.to_owned(),
        })
    }

    pub fn credential_handle(&self) -> &CredentialHandle {
        &self.credential_handle
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn responses_url(&self) -> &'static str {
        OPENAI_RESPONSES_URL
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OpenAiConfigError {
    #[error("OpenAI credentials must use a cred:openai.* handle")]
    InvalidCredentialHandle,
    #[error("OpenAI model must not be empty")]
    EmptyModel,
    #[error("OpenAI model exceeds 128 characters")]
    ModelTooLong,
    #[error("OpenAI model contains unsupported characters")]
    InvalidModel,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle(value: &str) -> CredentialHandle {
        match CredentialHandle::parse(value.to_owned()) {
            Ok(handle) => handle,
            Err(error) => panic!("fixture handle failed: {error}"),
        }
    }

    #[test]
    fn provider_origin_is_fixed_to_openai() {
        let config = OpenAiConfig::new(handle("cred:openai.default"), "gpt-5.6-sol");
        assert_eq!(
            config.ok().map(|value| value.responses_url()),
            Some(OPENAI_RESPONSES_URL)
        );
        assert_eq!(OPENAI_API_ORIGIN, "https://api.openai.com");
        assert_eq!(OPENAI_RESPONSES_PATH, "/v1/responses");
    }

    #[test]
    fn non_openai_credential_handles_are_rejected() {
        for value in [
            "cred:anthropic.default",
            "cred:gemini.default",
            "cred:xai.default",
            "cred:ollama.local",
            "cred:azure.openai",
        ] {
            let config = OpenAiConfig::new(handle(value), "gpt-5.6-sol");
            assert!(matches!(
                config,
                Err(OpenAiConfigError::InvalidCredentialHandle)
            ));
        }
    }

    #[test]
    fn no_arbitrary_endpoint_is_part_of_the_configuration() {
        let config = OpenAiConfig::new(handle("cred:openai.default"), "gpt-5.6-sol");
        match config {
            Ok(config) => assert_eq!(config.responses_url(), "https://api.openai.com/v1/responses"),
            Err(error) => panic!("unexpected OpenAI config error: {error}"),
        }
    }
}
