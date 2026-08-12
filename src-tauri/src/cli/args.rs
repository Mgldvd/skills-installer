use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "skills-installer",
    version,
    about = "Discover, configure, and install agent skills"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Path to a configuration file, overriding the normal precedence chain.
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Enable verbose structured logging.
    #[arg(long, global = true)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Launch the GUI (same as running with no subcommand at all).
    Gui,

    /// List configured and/or installed skills.
    List {
        /// Only show skills that are actually installed on disk.
        #[arg(long)]
        installed: bool,
        /// Print machine-readable JSON instead of a human-readable table.
        #[arg(long)]
        json: bool,
    },

    /// Install skills. With no ids and no --all, installs the currently
    /// preselected skills.
    Install {
        /// Specific configured skill ids to install.
        skill_ids: Vec<String>,

        /// Install every enabled configured skill.
        #[arg(long)]
        all: bool,

        /// Preview the commands that would run without executing anything.
        #[arg(long)]
        dry_run: bool,

        /// Skip the interactive confirmation prompt.
        #[arg(long)]
        yes: bool,

        /// Agent to install for (passed through to the Skills CLI).
        #[arg(long)]
        agent: Option<String>,

        /// Install into the global (user) scope instead of the project.
        #[arg(long, conflicts_with = "project")]
        global: bool,

        /// Install into the project scope (default).
        #[arg(long)]
        project: bool,

        /// Copy skill files instead of symlinking.
        #[arg(long)]
        copy: bool,

        /// Stop at the first failure instead of continuing with the rest.
        #[arg(long)]
        stop_on_error: bool,

        /// Print the final result as machine-readable JSON.
        #[arg(long)]
        json: bool,
    },

    /// Check Skills CLI availability and configuration health.
    Doctor {
        #[arg(long)]
        json: bool,
    },

    /// Print version information.
    Version,
}
