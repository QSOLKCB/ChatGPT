use std::error::Error;

use clap::{Parser, Subcommand};
use qsol_chatgpt::contracts::{Approval, CredentialHandle, ProposedAction};
use qsol_chatgpt::obs::{ObsConnectionConfig, DEFAULT_OBS_PORT};
use qsol_chatgpt::secrets::{SecretStore, SecretValue};
use qsol_chatgpt::{policy, runtime::Runtime, tui};

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
        #[arg(long, default_value_t = DEFAULT_OBS_PORT)]
        obs_port: u16,
        #[arg(long)]
        obs_password_env: Option<String>,
        #[arg(long, default_value = "cred:obs.main")]
        obs_credential_handle: String,
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
            obs_port,
            obs_password_env,
            obs_credential_handle,
        } => {
            let action = parse_action(&action)?;
            let approval = approve.then(|| Approval::allow_once(&action, "local-human"));
            let runtime = if execute && action.kind().starts_with("obs.") {
                let (config, secrets) = obs_runtime_config(
                    obs_port,
                    obs_password_env.as_deref(),
                    &obs_credential_handle,
                )?;
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

fn parse_action(json: &str) -> Result<qsol_chatgpt::contracts::Action, Box<dyn Error>> {
    let proposal: ProposedAction = serde_json::from_str(json)?;
    Ok(proposal.normalize()?)
}

fn obs_runtime_config(
    port: u16,
    password_env: Option<&str>,
    handle_text: &str,
) -> Result<(ObsConnectionConfig, SecretStore), Box<dyn Error>> {
    let mut secrets = SecretStore::default();
    let credential_handle = match password_env {
        Some(environment_name) => {
            if environment_name.trim().is_empty() {
                return Err("OBS password environment-variable name must not be empty".into());
            }
            let password = std::env::var(environment_name)?;
            let handle = CredentialHandle::parse(handle_text.to_owned())?;
            secrets.insert(handle.clone(), SecretValue::new(password));
            Some(handle)
        }
        None => None,
    };
    let config = ObsConnectionConfig::localhost(port, credential_handle)?;
    Ok((config, secrets))
}
