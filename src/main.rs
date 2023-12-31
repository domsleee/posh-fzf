use args::{Commands, RootArgs};
use clap::Parser;
use std::io::{self, Read};
use std::process::{Child, Command, Stdio};
mod args;
mod history;

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

pub fn wait_for_child<F>(_args: &RootArgs, child: &mut Child, formatter: F) -> io::Result<()>
where
    F: Fn(&str) -> String,
{
    let mut output: String = String::new();
    child.stdout.take().unwrap().read_to_string(&mut output)?;

    let status = child.wait()?;

    if status.success() {
        let output = formatter(output.trim());
        print!("{output}");
    } else {
        std::process::exit(1);
    }
    Ok(())
}

pub fn get_height(args: &RootArgs) -> String {
    args.height.clone().unwrap_or("45%".to_string())
}
