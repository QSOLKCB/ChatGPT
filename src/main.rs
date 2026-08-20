use std::error::Error;

use clap::{Parser, Subcommand};
use qsol_chatgpt::contracts::{Action, Approval, ProposedAction};
use qsol_chatgpt::obs::ObsConnectionConfig;
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
        #[arg(long)]
        obs_password_env: Option<String>,
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
            obs_password_env,
        } => {
            let action = parse_action(&action)?;
            let approval = approve.then(|| Approval::allow_once(&action, "local-human"));
            let runtime = if execute && action.kind().starts_with("obs.") {
                let (config, secrets) =
                    obs_runtime_config(&action, obs_password_env.as_deref())?;
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

fn obs_runtime_config(
    action: &Action,
    password_env: Option<&str>,
) -> Result<(ObsConnectionConfig, SecretStore), Box<dyn Error>> {
    let credential_handle = match action.credential_handles() {
        [] => None,
        [handle] => Some(handle.clone()),
        _ => return Err("OBS actions may bind at most one credential handle".into()),
    };

    let mut secrets = SecretStore::default();
    if let Some(environment_name) = password_env {
        if environment_name.trim().is_empty() {
            return Err("OBS password environment-variable name must not be empty".into());
        }
        let Some(handle) = credential_handle.as_ref() else {
            return Err("OBS password use requires the action to bind one credential handle".into());
        };
        let password = std::env::var(environment_name)?;
        secrets.insert(handle.clone(), SecretValue::new(password));
    }

    let config = ObsConnectionConfig::for_action(action, credential_handle)?;
    Ok((config, secrets))
}
