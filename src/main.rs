use std::error::Error;

use clap::{Parser, Subcommand};
use qsol_chatgpt::contracts::{Approval, ProposedAction};
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
        } => {
            let action = parse_action(&action)?;
            let approval = approve.then(|| Approval::allow_once(&action, "local-human"));
            let runtime = if execute {
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
