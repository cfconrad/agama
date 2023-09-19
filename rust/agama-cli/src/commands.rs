use crate::config::ConfigCommands;
use crate::profile::ProfileCommands;
use crate::questions::QuestionsCommands;
use crate::wicked::WickedCommands;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Change or show installation settings
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Display information about installation settings (e.g., possible values)
    Info {
        /// Configuration keys (e.g., software.products)
        keys: Vec<String>,
    },
    /// Start probing
    Probe,
    // Start Installation
    Install,
    /// Autoinstallation profile handling
    #[command(subcommand)]
    Profile(ProfileCommands),
    /// Configuration for questions that come from installer
    ///
    /// Questions are raised when an unexpected (by the user) situation happens in the installer:
    /// like if an encrypted partition is detected and cannot be inspected,
    /// if a repository is signed by an unknown GPG key, or if the installer is not sure
    /// if multipath should be activated.
    ///
    /// For more details see official agama documentation for Questions.
    #[command(subcommand)]
    Questions(QuestionsCommands),
    /// Migrate wicked config
    #[command(subcommand)]
    Wicked(WickedCommands),
}
