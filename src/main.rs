use clap::Parser;
use posh_fzf::args::{Commands, RootArgs};
use posh_fzf::history;
use posh_fzf::util::{get_height, wait_for_child};
use std::io::{self};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    let args = match RootArgs::try_parse() {
        Ok(args) => args,
        Err(e) => {
            let _ = e.print();
            std::process::exit(1);
        }
    };
    match args.command {
        Commands::Init => init(),
        Commands::History { ref history_path } => history::history(&args, history_path)?,
        Commands::PrintHistoryLine { history_line } => history::print_history_line(&history_line),
        Commands::Custom { ref trail_args } => custom(&args, trail_args)?,
        Commands::Fzf { ref trail_args } => fzf(&args, trail_args)?,
    };
    Ok(())
}

static INIT_DATA: &[u8] = include_bytes!("../resource/posh-fzf.ps1");
fn init() {
    println!(
        "{}",
        std::str::from_utf8(INIT_DATA).expect("able to get from utf-8")
    );
}

fn fzf(args: &RootArgs, trail_args: &[String]) -> io::Result<()> {
    let mut new_trail_args = vec!["fzf".to_string(), "--height".to_string(), get_height(args)];
    new_trail_args.extend(trail_args.to_owned());
    custom(args, &new_trail_args)
}

fn custom(args: &RootArgs, trail_args: &[String]) -> io::Result<()> {
    let mut child = Command::new(&trail_args[0])
        .args(trail_args.iter().skip(1))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()?;
    wait_for_child(args, &mut child, |x| x.to_string())
}
