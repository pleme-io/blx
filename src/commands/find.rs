use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct FindArgs {
    #[command(subcommand)]
    command: FindCommand,
}

#[derive(Subcommand)]
enum FindCommand {
    /// Find files by name pattern
    Files {
        /// Search query
        #[arg(default_value = "")]
        query: String,
    },
    /// Find directories by name pattern
    Dir {
        /// Search query
        #[arg(default_value = "")]
        query: String,
    },
    /// Search file contents with context
    Content {
        /// Search pattern
        pattern: String,
        /// Lines of context
        #[arg(short, long, default_value = "3")]
        context: usize,
    },
    /// Find a file and open in editor
    Edit {
        /// Search query
        #[arg(default_value = "")]
        query: String,
    },
    /// Interactively select and kill a process
    Kill,
    /// Interactively select a git branch and checkout
    Checkout,
}

impl FindArgs {
    pub fn run(self) -> Result<()> {
        match self.command {
            FindCommand::Files { query } => find_files(&query),
            FindCommand::Dir { query } => find_dirs(&query),
            FindCommand::Content { pattern, context } => find_content(&pattern, context),
            FindCommand::Edit { query } => find_edit(&query),
            FindCommand::Kill => find_kill(),
            FindCommand::Checkout => find_checkout(),
        }
    }
}

fn find_files(query: &str) -> Result<()> {
    let mut cmd = if which::which("fd").is_ok() {
        let mut c = Command::new("fd");
        c.arg("--type").arg("f").arg("--hidden").arg("--follow").arg("--exclude").arg(".git");
        if !query.is_empty() {
            c.arg(query);
        }
        c
    } else {
        let mut c = Command::new("find");
        c.arg(".").arg("-type").arg("f");
        if !query.is_empty() {
            c.arg("-name").arg(format!("*{query}*"));
        }
        c
    };
    cmd.status().context("failed to find files")?;
    Ok(())
}

fn find_dirs(query: &str) -> Result<()> {
    let mut cmd = if which::which("fd").is_ok() {
        let mut c = Command::new("fd");
        c.arg("--type").arg("d").arg("--hidden").arg("--follow").arg("--exclude").arg(".git");
        if !query.is_empty() {
            c.arg(query);
        }
        c
    } else {
        let mut c = Command::new("find");
        c.arg(".").arg("-type").arg("d");
        if !query.is_empty() {
            c.arg("-name").arg(format!("*{query}*"));
        }
        c
    };
    cmd.status().context("failed to find directories")?;
    Ok(())
}

fn find_content(pattern: &str, context: usize) -> Result<()> {
    let status = if which::which("rg").is_ok() {
        Command::new("rg")
            .arg("--color=always")
            .arg("--context")
            .arg(context.to_string())
            .arg(pattern)
            .status()
            .context("failed to run rg")?
    } else {
        Command::new("grep")
            .arg("-rn")
            .arg(format!("-C{context}"))
            .arg(pattern)
            .arg(".")
            .status()
            .context("failed to run grep")?
    };
    if !status.success() && status.code() != Some(1) {
        anyhow::bail!("search failed with status {status}");
    }
    Ok(())
}

fn find_edit(query: &str) -> Result<()> {
    // Pipe fd output through skim, then open in editor
    let fd_cmd = if which::which("fd").is_ok() {
        format!("fd --type f --hidden --follow --exclude .git {query}")
    } else {
        format!("find . -type f -name '*{query}*'")
    };

    let selector = if which::which("sk").is_ok() { "sk" } else { "fzf" };
    let preview = if which::which("bat").is_ok() {
        "--preview 'bat --color=always {}' --preview-window=right:60%"
    } else {
        ""
    };

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());
    let script = format!("file=$({fd_cmd} | {selector} {preview}) && [ -n \"$file\" ] && {editor} \"$file\"");

    Command::new("sh")
        .arg("-c")
        .arg(&script)
        .status()
        .context("failed to run find-edit")?;
    Ok(())
}

fn find_kill() -> Result<()> {
    let selector = if which::which("sk").is_ok() { "sk" } else { "fzf" };
    let script = format!(
        "pid=$(ps aux | sed 1d | {selector} -m | awk '{{print $2}}') && [ -n \"$pid\" ] && echo $pid | xargs kill -9"
    );
    Command::new("sh")
        .arg("-c")
        .arg(&script)
        .status()
        .context("failed to run find-kill")?;
    Ok(())
}

fn find_checkout() -> Result<()> {
    let selector = if which::which("sk").is_ok() { "sk" } else { "fzf" };
    let script = format!(
        "branch=$(git branch --all | grep -v HEAD | {selector} | sed 's/^[* ]*//' | sed 's#remotes/origin/##') && [ -n \"$branch\" ] && git checkout \"$branch\""
    );
    Command::new("sh")
        .arg("-c")
        .arg(&script)
        .status()
        .context("failed to run find-checkout")?;
    Ok(())
}
