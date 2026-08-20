use std::error::Error;
use std::io::{self, IsTerminal, Read};

use clap::{Parser, Subcommand};
use qsol_chatgpt::contracts::{Action, Approval, ProposedAction};
use qsol_chatgpt::obs::ObsConnectionConfig;
use qsol_chatgpt::policy::Disposition;
use qsol_chatgpt::secrets::{SecretStore, SecretValue};
use qsol_chatgpt::{policy, runtime::Runtime, tui};
use zeroize::Zeroize;

const MAX_OBS_PASSWORD_BYTES: usize = 4096;

#[derive(Parser)]
#[command(name = "qsol-chatgpt", version, about = "Linux-native AI capability broker")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Tui,
    Policy {
        action: String,
    },
    Run {
        action: String,
        #[arg(long)]
        approve: bool,
        #[arg(long)]
        execute: bool,
        #[arg(long)]
        obs_password_stdin: bool,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => tui::run()?,
        Commands::Policy { action } => {
            let action = parse_action(&action)?;
            println!("{}", serde_json::to_string_pretty(&policy::evaluate(&action))?);
        }
        Commands::Run {
            action,
            approve,
            execute,
            obs_password_stdin,
        } => {
            let action = parse_action(&action)?;
            let approval = approve.then(|| Approval::allow_once(&action, "local-human"));
            let runtime = if needs_obs_runtime(&action, execute) {
                let (config, secrets) = obs_runtime_config(&action, obs_password_stdin)?;
                Runtime::effectful_with_obs(config, secrets)
            } else if execute {
                Runtime::effectful()
            } else {
                Runtime::simulated()
            };
            let receipt = runtime.run(&action, approval.as_ref())?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
    }
    Ok(())
}

fn parse_action(json: &str) -> Result<Action, Box<dyn Error>> {
    let proposal: ProposedAction = serde_json::from_str(json)?;
    Ok(proposal.normalize()?)
}

fn needs_obs_runtime(action: &Action, execute: bool) -> bool {
    execute
        && action.kind().starts_with("obs.")
        && policy::evaluate(action).disposition != Disposition::Deny
}

fn obs_runtime_config(
    action: &Action,
    password_stdin: bool,
) -> Result<(ObsConnectionConfig, SecretStore), Box<dyn Error>> {
    let credential_handle = match action.credential_handles() {
        [] => None,
        [handle] => Some(handle.clone()),
        _ => return Err("OBS actions may bind at most one credential handle".into()),
    };

    let mut secrets = SecretStore::default();
    if password_stdin {
        let Some(handle) = credential_handle.as_ref() else {
            return Err("OBS password use requires the action to bind one credential handle".into());
        };
        let password = read_obs_password_stdin()?;
        secrets.insert(handle.clone(), SecretValue::new(password));
    }

    let config = ObsConnectionConfig::for_action(action, credential_handle)?;
    Ok((config, secrets))
}

fn read_obs_password_stdin() -> Result<String, Box<dyn Error>> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        return Err(
            "--obs-password-stdin requires redirected or piped stdin; interactive terminal input is refused"
                .into(),
        );
    }
    let mut locked = stdin.lock();
    read_bounded_secret(&mut locked)
}

fn read_bounded_secret<R: Read>(reader: &mut R) -> Result<String, Box<dyn Error>> {
    let mut bytes = Vec::new();
    let read_result = reader
        .take((MAX_OBS_PASSWORD_BYTES + 2) as u64)
        .read_to_end(&mut bytes);
    if let Err(error) = read_result {
        bytes.zeroize();
        return Err(error.into());
    }

    let mut end = bytes.len();
    if end > 0 && bytes[end - 1] == b'\n' {
        end -= 1;
    }
    if end > 0 && bytes[end - 1] == b'\r' {
        end -= 1;
    }
    if end == 0 || end > MAX_OBS_PASSWORD_BYTES {
        bytes.zeroize();
        return Err("OBS password stdin payload must be 1..=4096 UTF-8 bytes".into());
    }

    let password = match std::str::from_utf8(&bytes[..end]) {
        Ok(value) => value.to_owned(),
        Err(error) => {
            bytes.zeroize();
            return Err(error.into());
        }
    };
    bytes.zeroize();
    Ok(password)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn action(json: &str) -> Action {
        match parse_action(json) {
            Ok(value) => value,
            Err(error) => panic!("fixture action failed: {error}"),
        }
    }

    #[test]
    fn denied_obs_actions_do_not_require_runtime_configuration() {
        let denied = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.stream.start","args":{"obs_port":4455}}"#,
        );
        assert!(!needs_obs_runtime(&denied, true));

        let allowed = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.scene.list","args":{"obs_port":4455}}"#,
        );
        assert!(needs_obs_runtime(&allowed, true));
    }

    #[test]
    fn protected_secret_reader_accepts_piped_line_and_trims_transport_newline() {
        let mut input = Cursor::new(b"obs-secret\r\n".to_vec());
        let result = read_bounded_secret(&mut input);
        assert_eq!(result.ok().as_deref(), Some("obs-secret"));
    }

    #[test]
    fn protected_secret_reader_rejects_oversized_payload() {
        let mut input = Cursor::new(vec![b'x'; MAX_OBS_PASSWORD_BYTES + 1]);
        assert!(read_bounded_secret(&mut input).is_err());
    }
}
