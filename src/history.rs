use std::fs::File;
use std::io::Write;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{ChildStdin, Command, Stdio};

use crate::args::RootArgs;
use crate::timings::is_timings_enabled;
use crate::timings::{TIMINGS, write_perf_logs};
use crate::util::{get_height, wait_for_child};
use crate::{timing_end, timing_start};
const HISTORY_NEWLINE_CHAR: char = '↵';

pub fn history(args: &RootArgs, history_path: &PathBuf) -> io::Result<()> {
    timing_start!("child_fork");
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
    timing_end!("child_fork");

    timing_start!("get_stdin");
    let fzf_stdin = child.stdin.as_mut().expect("Failed to open stdin");
    timing_end!("get_stdin");

    timing_start!("write_history_to_fzf_stdin");
    write_history_to_fzf_stdin(fzf_stdin, history_path)?;
    timing_end!("write_history_to_fzf_stdin");

    write_perf_logs()?;
    wait_for_child(args, &mut child, |x| x.replace(HISTORY_NEWLINE_CHAR, "\n"))
}

fn write_history_to_fzf_stdin(
    fzf_stdin: &mut ChildStdin,
    history_path: &PathBuf,
) -> io::Result<()> {
    let history_set = get_history_recent_commands(history_path)?;
    let all_data = history_set.join("\n") + "\n";
    fzf_stdin.write_all(all_data.as_bytes())?;
    Ok(())
}

pub fn print_history_line(history_line: &str) {
    println!("{}", history_line.replace(HISTORY_NEWLINE_CHAR, "\n"))
}

/// Get historical commands in most recent order
pub fn get_history_recent_commands(history_path: &PathBuf) -> io::Result<Vec<String>> {
    let all_lines = get_history_all_commands(history_path)?;
    Ok(get_unique_reversed(all_lines))
}

pub fn get_history_all_commands(history_path: &PathBuf) -> io::Result<Vec<String>> {
    timing_start!("get_history_all_commands");
    let file = File::open(history_path)?;
    let mut reader = io::BufReader::new(file);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;
    let mut all_lines = Vec::new();
    let mut current_line = String::new();
    for line in buffer.lines() {
        if line.ends_with('`') {
            current_line.push_str(line.trim_end_matches('`'));
            current_line.push(HISTORY_NEWLINE_CHAR);
        } else {
            current_line.push_str(line);
            all_lines.push(current_line);
            current_line = String::new();
        }
    }

    if !current_line.is_empty() {
        all_lines.push(current_line);
    }
    timing_end!("get_history_all_commands");
    Ok(all_lines)
}

pub fn get_unique_reversed(all_lines: Vec<String>) -> Vec<String> {
    timing_start!("get_unique_reversed");
    let mut set = indexmap::IndexSet::with_hasher(ahash::RandomState::new());
    set.extend(all_lines.into_iter().rev());
    let res = set.into_iter().collect::<Vec<_>>();
    timing_end!("get_unique_reversed");
    res
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
                &format!("multi{HISTORY_NEWLINE_CHAR}line"),
                "3rd",
                "4th"
            ]
        );
    }
}
