use std::env::{self};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::{ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

use crate::args::RootArgs;
use crate::{get_height, wait_for_child};
const HISTORY_NEWLINE: &str = "↵";

pub fn history(args: &RootArgs, history_path: &PathBuf) -> io::Result<()> {
    let child_start = Instant::now();
    let mut child = Command::new("fzf")
        .arg("--height")
        .arg(get_height(args))
        .arg("--scheme=history")
        .arg("--preview")
        .arg("posh-fzf print-history-line {}")
        .arg("--preview-window=right:30%")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let child_duration = child_start.elapsed();

    let stdin_start = Instant::now();
    let fzf_stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let stdin_duration = stdin_start.elapsed();

    let write_history_instant = Instant::now();
    write_history_to_fzf_stdin(fzf_stdin, history_path)?;
    let write_history_duration = write_history_instant.elapsed();

    write_perf_logs(child_duration, stdin_duration, write_history_duration)?;
    wait_for_child(args, &mut child, |x| x.replace(HISTORY_NEWLINE, "\n"))
}

fn write_history_to_fzf_stdin(
    fzf_stdin: &mut ChildStdin,
    history_path: &PathBuf,
) -> io::Result<()> {
    let history_set = get_history_recent_commands(history_path)?;
    let all_data = history_set.join("\n") + "\n";
    let mut buffered_stdin = BufWriter::with_capacity(256 * 1024, fzf_stdin); // Use a large buffer (256KB)
    buffered_stdin.write_all(all_data.as_bytes())?;
    Ok(())
}

pub fn print_history_line(history_line: &str) {
    println!("{}", history_line.replace(HISTORY_NEWLINE, "\n"))
}

/// Get historical commands in most recent order
pub fn get_history_recent_commands(history_path: &PathBuf) -> io::Result<Vec<String>> {
    let file = File::open(history_path)?;
    let reader = io::BufReader::new(file);
    let mut all_lines: Vec<String> = Vec::new();
    let mut current_line = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.ends_with('`') {
            current_line.push_str(line.trim_end_matches('`'));
            current_line.push_str(HISTORY_NEWLINE);
        } else {
            current_line.push_str(&line);
            all_lines.push(current_line);
            current_line = String::new();
        }
    }

    if !current_line.is_empty() {
        all_lines.push(current_line);
    }

    let set: indexmap::IndexSet<_> = all_lines.into_iter().rev().collect();
    Ok(set.into_iter().collect())
}

fn write_perf_logs(
    child_duration: Duration,
    stdin_duration: Duration,
    write_history_duration: Duration,
) -> io::Result<()> {
    if env::var("POSH_FZF_PERF").is_err() {
        return Ok(());
    }
    let home = dirs_next::home_dir().expect("has home directory");
    let log_file_path = home.join("posh-fzf.log");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)?;

    writeln!(file, "New log")?;
    writeln!(file, "{child_duration:?}: child_duration")?;
    writeln!(file, "{stdin_duration:?}: stdin_duration")?;
    writeln!(file, "{write_history_duration:?}: write_history_duration")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_get_history_recent_commands() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_history.txt");
        let mut file = File::create(&file_path).unwrap();
        let content = r#"1st
4th
3rd
multi`
line
2nd
1st
"#;

        file.write_all(content.as_bytes())
            .expect("Failed to write to file");
        file.flush().expect("Failed to flush file");

        let history_lines = get_history_recent_commands(&file_path).unwrap();
        assert_eq!(
            history_lines,
            vec![
                "1st",
                "2nd",
                &format!("multi{HISTORY_NEWLINE}line"),
                "3rd",
                "4th"
            ]
        );
    }
}
