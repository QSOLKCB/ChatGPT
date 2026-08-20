use std::process::Command;

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::Zeroize;

use crate::contracts::Action;

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("executor does not support this action")]
    Unsupported,
    #[error("credential injection is not implemented")]
    CredentialsUnsupported,
    #[error("invalid argv")]
    InvalidArgv,
    #[error("process launch failed")]
    SpawnFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExecutionEvidence {
    pub exit_code: Option<i32>,
    pub stdout_sha256: String,
    pub stdout_bytes: usize,
    pub stderr_sha256: String,
    pub stderr_bytes: usize,
}

pub fn execute(action: &Action) -> Result<ExecutionEvidence, ExecutionError> {
    if action.kind() != "shell.exec" {
        return Err(ExecutionError::Unsupported);
    }
    if !action.credential_handles().is_empty() {
        return Err(ExecutionError::CredentialsUnsupported);
    }

    let Some(Value::Array(raw_argv)) = action.args().get("argv") else {
        return Err(ExecutionError::InvalidArgv);
    };
    let argv = raw_argv
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .ok_or(ExecutionError::InvalidArgv)?;
    let Some((program, args)) = argv.split_first() else {
        return Err(ExecutionError::InvalidArgv);
    };

    let mut output = Command::new(program)
        .args(args)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LC_ALL", "C")
        .output()
        .map_err(|_| ExecutionError::SpawnFailed)?;

    let evidence = ExecutionEvidence {
        exit_code: output.status.code(),
        stdout_sha256: format!("{:x}", Sha256::digest(&output.stdout)),
        stdout_bytes: output.stdout.len(),
        stderr_sha256: format!("{:x}", Sha256::digest(&output.stderr)),
        stderr_bytes: output.stderr.len(),
    };

    output.stdout.zeroize();
    output.stderr.zeroize();
    Ok(evidence)
}
