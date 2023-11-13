use std::fs::File;
use std::io::{self, BufRead, BufWriter};
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::{ChildStdin, Command, Stdio};

use crate::args::RootArgs;
use crate::{get_height, wait_for_child};
const HISTORY_NEWLINE: &str = "↵";

pub fn history(args: &RootArgs, history_path: &PathBuf) -> io::Result<()> {
    let mut child = Command::new("fzf")
        .arg("--height")
        .arg(get_height(args))
        .arg("--scheme=history")
        .arg("--preview")
        .arg("posh-fzf print-history-line {}")
        // .arg("--preview-window")
        // .arg("down")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let fzf_stdin = child.stdin.as_mut().expect("Failed to open stdin");
    write_history_to_fzf_stdin(fzf_stdin, history_path)?;

    wait_for_child(args, &mut child, |x| x.replace(HISTORY_NEWLINE, "\n"))
}

fn write_history_to_fzf_stdin(
    fzf_stdin: &mut ChildStdin,
    history_path: &PathBuf,
) -> io::Result<()> {
    let file = File::open(history_path)?;
    let reader = BufReader::new(file);
    let mut set = indexmap::IndexSet::new();
    let mut val: String = String::from("");
    for line in reader.lines() {
        let mut data = line?;
        if data.ends_with('`') {
            data.pop();
            val.push_str(&data);
            val.push_str(HISTORY_NEWLINE);
        } else {
            val.push_str(&data);
            set.insert(val.clone());
            val.clear();
        }
    }

    let mut all_data = String::new();
    for el in set.iter().rev() {
        all_data.push_str(&el);
        all_data.push('\n');
    }

    let mut writer = BufWriter::new(fzf_stdin);
    writer.write_all(all_data.as_bytes())?;
    writer.flush()?;

    Ok(())
}

pub fn print_history_line(history_line: &str) {
    println!("{}", history_line.replace(HISTORY_NEWLINE, "\n"))
}
