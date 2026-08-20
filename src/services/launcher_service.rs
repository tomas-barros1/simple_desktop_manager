use crate::models::DesktopEntry;
use std::process::Command;
use tracing::info;

#[derive(Debug, PartialEq, Eq)]
pub enum LaunchOutcome {
    UrlOpened(String),
    CommandLaunched(String),
}

/// Clean execution tokens from a .desktop Exec command line (e.g. `%u`, `%f`, `%F`, `%U`, `%i`, `%c`, `%k`).
pub fn clean_exec_command(exec: &str) -> String {
    let tokens: Vec<&str> = exec
        .split_whitespace()
        .filter(|s| !s.starts_with('%'))
        .collect();
    tokens.join(" ")
}

/// Launch a desktop entry (either opening its URL or executing its binary).
pub fn launch_entry(entry: &DesktopEntry) -> Result<LaunchOutcome, std::io::Error> {
    let exec_raw = entry.exec.trim();
    let url_raw = entry.url.trim();

    if exec_raw.is_empty() && !url_raw.is_empty() {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(url_raw);
        cmd.spawn()?;
        info!(url = url_raw, "opened URL");
        return Ok(LaunchOutcome::UrlOpened(url_raw.to_string()));
    }

    if exec_raw.is_empty() {
        return Err(std::io::Error::other("Exec command is empty"));
    }

    let cleaned = clean_exec_command(exec_raw);
    if cleaned.is_empty() {
        return Err(std::io::Error::other("No valid executable in command"));
    }

    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    let is_sudo = tokens.first() == Some(&"sudo");
    let cmd_str = if is_sudo {
        tokens[1..].join(" ")
    } else {
        cleaned.clone()
    };

    let mut command = Command::new("sh");
    if is_sudo {
        command.arg("-c").arg(format!("pkexec {cmd_str}"));
    } else {
        command.arg("-c").arg(&cmd_str);
    }

    let path_dir = entry.path.trim();
    if !path_dir.is_empty() {
        command.current_dir(path_dir);
    } else {
        // Fallback: if path is not specified, attempt to use the binary's parent directory
        let raw_bin = cmd_str.trim_matches('"').trim_matches('\'');
        let candidate = std::path::Path::new(raw_bin);
        if let Some(parent) = candidate.parent() {
            if parent.is_dir() && parent != std::path::Path::new("") && parent != std::path::Path::new("/") {
                command.current_dir(parent);
            }
        }
    }

    command.spawn()?;
    info!(cmd = %cleaned, "launched application command");
    Ok(LaunchOutcome::CommandLaunched(cleaned))
}
