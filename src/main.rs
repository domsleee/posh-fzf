use args::{Commands, RootArgs};
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Read};
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
mod args;

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
        Commands::SelectMultiple => select_multiple(&args)?,
        Commands::ChangeDirectory {} => change_directory(&args)?,
        Commands::History { ref history_path } => history(&args, history_path)?,
    };
    Ok(())
}

static INIT_DATA: &[u8] = include_bytes!("../resource/init.ps1");
fn init() {
    println!(
        "{}",
        std::str::from_utf8(INIT_DATA).expect("able to get from utf-8")
    );
}

fn select_multiple(args: &RootArgs) -> io::Result<()> {
    let mut child = Command::new("fzf")
        .arg("--height=11")
        .arg("-m")
        .env("FZF_DEFAULT_COMMAND", "fd   --hidden --exclude \".git\"")
        .stdout(Stdio::piped())
        .spawn()?;

    wait_for_child(args, &mut child, |x| format!("{x}"))
}

fn change_directory(args: &RootArgs) -> io::Result<()> {
    let mut child = Command::new("fzf")
        .arg("--height=11")
        .env(
            "FZF_DEFAULT_COMMAND",
            "fd --type d --hidden --exclude \".git\"",
        )
        .stdout(Stdio::piped())
        .spawn()?;

    wait_for_child(args, &mut child, |x| format!("cd '{x}'"))
}

fn history(args: &RootArgs, history_path: &PathBuf) -> io::Result<()> {
    let mut child = Command::new("fzf")
        .arg("--height=11")
        .arg("--scheme=history")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let fzf_stdin = child.stdin.as_mut().expect("Failed to open stdin");
    write_to_fzf_stdin(fzf_stdin, history_path)?;

    wait_for_child(args, &mut child, |x| format!("{x}"))
}

fn write_to_fzf_stdin(fzf_stdin: &mut ChildStdin, history_path: &PathBuf) -> io::Result<()> {
    let mut writer = BufWriter::new(fzf_stdin);
    let file = File::open(history_path)?;
    let reader = BufReader::new(file);
    let mut set = indexmap::IndexSet::new();

    let mut val: String = String::from("");
    for line in reader.lines() {
        let mut data = line?;
        if data.ends_with("`") {
            data.pop();
            val = val + &data;
        } else {
            set.insert(val + &data);
            val = String::from("");
        }
    }

    for el in set.iter().rev() {
        writeln!(writer, "{}", el)?;
    }
    writer.flush()?; // Flush the buffer to make sure everything gets written
    Ok(())
}

fn wait_for_child<F>(args: &RootArgs, child: &mut Child, formatter: F) -> io::Result<()>
where
    F: Fn(&str) -> String,
{
    let mut output: String = String::new();
    child.stdout.take().unwrap().read_to_string(&mut output)?;

    let status = child.wait()?;

    if status.success() {
        std::fs::write(
            &args.temp_file_name.clone().unwrap(),
            formatter(output.trim()),
        )
        .unwrap();
    }
    Ok(())
}
