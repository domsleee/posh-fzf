use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Fzf bindings for powershell
#[derive(Parser, Debug)]
pub struct RootArgs {
    #[command(subcommand)]
    pub command: Commands,

    /// Temporary file name to store the result (otherwise printed to console)
    #[arg(long)]
    pub temp_file_name: Option<PathBuf>,

    /// Number of rows to be passed as `--height` to fzf
    #[arg(long)]
    pub height: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Init powershell script
    Init,
    /// Reverse history search (ctrl+r)
    History { history_path: PathBuf },
    /// Custom command
    Custom {
        #[clap(last = true)]
        trail_args: Vec<String>,
    },
    /// Invoke fzf (short for `posh-fzf custom -- fzf --height <default>`)
    Fzf {
        #[clap(last = true)]
        trail_args: Vec<String>,
    },
}
