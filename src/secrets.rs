use std::collections::BTreeMap;
use std::fmt;

use zeroize::Zeroizing;

use crate::contracts::CredentialHandle;

pub struct SecretValue(Zeroizing<String>);

impl SecretValue {
    pub fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }

    pub fn with_exposed<R>(&self, operation: impl FnOnce(&str) -> R) -> R {
        operation(self.0.as_str())
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretValue([REDACTED])")
    }
}

#[derive(Default)]
pub struct SecretStore {
    values: BTreeMap<CredentialHandle, SecretValue>,
}

impl SecretStore {
    pub fn insert(&mut self, handle: CredentialHandle, value: SecretValue) {
        self.values.insert(handle, value);
    }

    pub fn with_secret<R>(
        &self,
        handle: &CredentialHandle,
        operation: impl FnOnce(&str) -> R,
    ) -> Option<R> {
        self.values.get(handle).map(|value| value.with_exposed(operation))
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl fmt::Debug for SecretStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretStore")
            .field("entries", &self.values.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_never_contains_secret() {
        let secret = SecretValue::new("sk-test-do-not-print".to_owned());
        let rendered = format!("{secret:?}");
        assert!(!rendered.contains("sk-test-do-not-print"));
        assert!(rendered.contains("REDACTED"));
    }
}
