use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead, BufWriter};
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
        .arg("--preview-window=right:30%")
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
    let history_set = get_history_recent_commands(history_path)?;

    let mut all_data = String::new();
    for el in history_set {
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
