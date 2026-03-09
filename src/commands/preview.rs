//! `blx preview` — fzf-tab preview helpers.
//!
//! Standalone binaries via symlinks: blx-preview, blx-preview-dir,
//! blx-preview-proc, blx-preview-git.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct PreviewArgs {
    #[command(subcommand)]
    command: PreviewCommand,
}

#[derive(Subcommand)]
pub enum PreviewCommand {
    /// Preview a file or directory (bat for files, eza for dirs)
    File {
        /// Path to preview
        path: String,
    },
    /// Preview a directory listing
    Dir {
        /// Directory path
        path: String,
    },
    /// Preview a process (for kill/ps completion)
    Proc {
        /// fzf-tab group
        group: String,
        /// Process identifier (PID or name)
        word: String,
    },
    /// Preview git context (diff, log, checkout)
    Git {
        /// Git subcommand context: diff, log, checkout
        context: String,
        /// The word being completed
        word: String,
        /// Optional group (for checkout)
        group: Option<String>,
    },
}

impl PreviewArgs {
    pub fn run(self) -> Result<()> {
        match self.command {
            PreviewCommand::File { path } => preview_file(&path),
            PreviewCommand::Dir { path } => preview_dir(&path),
            PreviewCommand::Proc { group, word } => preview_proc(&group, &word),
            PreviewCommand::Git { context, word, group } => {
                preview_git(&context, &word, group.as_deref())
            }
        }
    }
}

/// Preview a file (bat) or directory (eza). Fallback: print the path.
pub fn preview_file(path: &str) -> Result<()> {
    let p = std::path::Path::new(path);
    if p.is_dir() {
        return preview_dir(path);
    }
    if p.is_file() {
        let status = Command::new("bat")
            .args(["--color=always", "--style=numbers", "--line-range=:200", path])
            .status()
            .context("failed to run bat")?;
        if !status.success() {
            println!("{path}");
        }
        return Ok(());
    }
    println!("{path}");
    Ok(())
}

/// Preview a directory with eza.
pub fn preview_dir(path: &str) -> Result<()> {
    let status = Command::new("eza")
        .args(["-1", "--color=always", "--icons", "--group-directories-first", path])
        .status()
        .context("failed to run eza")?;
    if !status.success() {
        println!("{path}");
    }
    Ok(())
}

/// Preview a process (for kill/ps completion).
pub fn preview_proc(group: &str, word: &str) -> Result<()> {
    if group == "[process ID]" {
        let _ = Command::new("ps")
            .args(["-p", word, "-o", "comm,pid,ppid,%cpu,%mem,start,time,command"])
            .status();
    }
    Ok(())
}

/// Preview git context (diff, log, checkout).
pub fn preview_git(context: &str, word: &str, group: Option<&str>) -> Result<()> {
    match context {
        "diff" => {
            let diff = Command::new("git")
                .args(["diff", word])
                .output()
                .context("failed to run git diff")?;
            if diff.status.success() && !diff.stdout.is_empty() {
                if which::which("delta").is_ok() {
                    let mut delta = Command::new("delta")
                        .arg("--width=80")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                        .context("failed to run delta")?;
                    if let Some(stdin) = delta.stdin.as_mut() {
                        use std::io::Write;
                        let _ = stdin.write_all(&diff.stdout);
                    }
                    let _ = delta.wait();
                } else {
                    print!("{}", String::from_utf8_lossy(&diff.stdout));
                }
            }
        }
        "log" => {
            let _ = Command::new("git")
                .args(["log", "--oneline", "--graph", "--color=always", word])
                .status();
        }
        "checkout" => {
            let g = group.unwrap_or("");
            if g == "modified file" {
                return preview_git("diff", word, None);
            }
            let _ = Command::new("git")
                .args(["log", "--oneline", "--graph", "--color=always", word])
                .status();
        }
        _ => {
            println!("{context} {word}");
        }
    }
    Ok(())
}
