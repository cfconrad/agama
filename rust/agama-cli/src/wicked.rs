use crate::error::CliError;
use crate::printers::{print, Format};
use std::io;
use clap::Subcommand;
use agama_migrate_wicked::reader::read_dir as wicked_read_dir;
use agama_migrate_wicked::migrate::migrate;

#[derive(Subcommand, Debug)]
pub enum WickedCommands {
    /// Shows the current xml wicked configuration
    Show { path: String },
    /// Migrate wicked state at path
    Migrate { path: String },
}

pub enum WickedAction {
    Show(String),
    Migrate(String),
}

pub async fn run(subcommand: WickedCommands, format: Format) -> anyhow::Result<()> {
    let command = parse_wicked_command(subcommand)?;
    match command {
        WickedAction::Show(path) => {
            let interfaces = wicked_read_dir(path).await?;
            print(interfaces, io::stdout(), format)?;
            Ok(())
        },
        WickedAction::Migrate(path) => {
            let interfaces = wicked_read_dir(path).await?;
            migrate(interfaces).await;
            Ok(())
        }
    }
}

fn parse_wicked_command(subcommand: WickedCommands) -> Result<WickedAction, CliError> {
    match subcommand {
        WickedCommands::Show { path } => Ok(WickedAction::Show(path)),
        WickedCommands::Migrate { path } => Ok(WickedAction::Migrate(path)),
    }
}
