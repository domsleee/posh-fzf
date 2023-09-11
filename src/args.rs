use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Fzf bindings for powershell
#[derive(Parser, Debug)]
pub struct RootArgs {
    #[command(subcommand)]
    pub command: Commands,

    // named pipe
    #[arg(long)]
    pub temp_file_name: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Init powershell script
    Init,
    /// Select multiple paths
    SelectMultiple,
    /// Change directory (alt+c)
    ChangeDirectory {},
    /// Reverse history search (ctrl+r)
    History { history_path: PathBuf },
}
