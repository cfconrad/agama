use clap::Parser;

mod commands;
mod config;
mod error;
mod printers;
mod profile;
mod progress;
mod questions;
mod wicked;

use crate::error::CliError;
use agama_lib::error::ServiceError;
use agama_lib::manager::ManagerClient;
use agama_lib::progress::ProgressMonitor;
use async_std::task::{self, block_on};
use commands::Commands;
use config::run as run_config_cmd;
use printers::Format;
use profile::run as run_profile_cmd;
use progress::InstallerProgress;
use questions::run as run_questions_cmd;
use std::{
    process::{ExitCode, Termination},
    thread::sleep,
    time::Duration,
};
use wicked::run as run_wicked_cmd;

#[derive(Parser)]
#[command(name = "agama", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Format output
    #[arg(value_enum, short, long, default_value_t = Format::Json)]
    pub format: Format,
}

async fn probe() -> anyhow::Result<()> {
    let another_manager = build_manager().await?;
    let probe = task::spawn(async move { another_manager.probe().await });
    show_progress().await?;

    Ok(probe.await?)
}

/// Starts the installation process
///
/// Before starting, it makes sure that the manager is idle.
///
/// * `manager`: the manager client.
async fn install(manager: &ManagerClient<'_>, max_attempts: u8) -> anyhow::Result<()> {
    if manager.is_busy().await {
        println!("Agama's manager is busy. Waiting until it is ready...");
    }

    // Make sure that the manager is ready
    manager.wait().await?;

    if !manager.can_install().await? {
        return Err(CliError::ValidationError)?;
    }

    let progress = task::spawn(async { show_progress().await });
    // Try to start the installation up to max_attempts times.
    let mut attempts = 1;
    loop {
        match manager.install().await {
            Ok(()) => break,
            Err(e) => {
                eprintln!(
                    "Could not start the installation process: {e}. Attempt {}/{}.",
                    attempts, max_attempts
                );
            }
        }
        if attempts == max_attempts {
            eprintln!("Giving up.");
            return Err(CliError::InstallationError)?;
        }
        attempts += 1;
        sleep(Duration::from_secs(1));
    }
    let _ = progress.await;
    Ok(())
}

async fn show_progress() -> Result<(), ServiceError> {
    // wait 1 second to give other task chance to start, so progress can display something
    task::sleep(Duration::from_secs(1)).await;
    let conn = agama_lib::connection().await?;
    let mut monitor = ProgressMonitor::new(conn).await.unwrap();
    let presenter = InstallerProgress::new();
    monitor
        .run(presenter)
        .await
        .expect("failed to monitor the progress");
    Ok(())
}

async fn wait_for_services(manager: &ManagerClient<'_>) -> Result<(), ServiceError> {
    let services = manager.busy_services().await?;
    // TODO: having it optional
    if !services.is_empty() {
        eprintln!("The Agama service is busy. Waiting for it to be available...");
        show_progress().await?
    }
    Ok(())
}

async fn build_manager<'a>() -> anyhow::Result<ManagerClient<'a>> {
    let conn = agama_lib::connection().await?;
    Ok(ManagerClient::new(conn).await?)
}

async fn run_command(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Config(subcommand) => {
            let manager = build_manager().await?;
            block_on(wait_for_services(&manager))?;
            block_on(run_config_cmd(subcommand, cli.format))
        }
        Commands::Probe => {
            let manager = build_manager().await?;
            block_on(wait_for_services(&manager))?;
            block_on(probe())
        }
        Commands::Profile(subcommand) => Ok(run_profile_cmd(subcommand)?),
        Commands::Install => {
            let manager = build_manager().await?;
            block_on(install(&manager, 3))
        }
        Commands::Questions(subcommand) => block_on(run_questions_cmd(subcommand)),
        Commands::Wicked(subcommand) => block_on(run_wicked_cmd(subcommand, cli.format)),
        _ => unimplemented!(),
    }
}

/// Represents the result of execution.
pub enum CliResult {
    /// Successful execution.
    Ok = 0,
    /// Something went wrong.
    Error = 1,
}

impl Termination for CliResult {
    fn report(self) -> ExitCode {
        ExitCode::from(self as u8)
    }
}

#[async_std::main]
async fn main() -> CliResult {
    let cli = Cli::parse();

    if let Err(error) = run_command(cli).await {
        eprintln!("{:?}", error);
        return CliResult::Error;
    }
    CliResult::Ok
}
