use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct GitArgs {
    #[command(subcommand)]
    command: GitCommand,
}

#[derive(Subcommand)]
enum GitCommand {
    /// Stage all changes and commit
    Commit {
        /// Commit message
        message: String,
    },
    /// Stage all, commit, and push
    Push {
        /// Commit message
        message: String,
    },
    /// Clone a repository
    Clone {
        /// Repository URL or owner/repo shorthand
        repo: String,
    },
    /// Commit with ISO-8601 timestamp in message
    Timestamp {
        /// Commit message (timestamp appended)
        message: String,
    },
    /// Delete branches that have been merged into the current branch
    CleanBranches,
    /// Show a compact, decorated git log tree
    Tree {
        /// Number of commits to show
        #[arg(short, long, default_value = "25")]
        count: usize,
    },
}

impl GitArgs {
    pub fn run(self) -> Result<()> {
        match self.command {
            GitCommand::Commit { message } => git_commit(&message, false),
            GitCommand::Push { message } => git_commit(&message, true),
            GitCommand::Clone { repo } => git_clone(&repo),
            GitCommand::Timestamp { message } => git_timestamp(&message),
            GitCommand::CleanBranches => git_clean_branches(),
            GitCommand::Tree { count } => git_tree(count),
        }
    }
}

fn git_commit(message: &str, push: bool) -> Result<()> {
    let status = Command::new("git")
        .args(["add", "--all"])
        .status()
        .context("failed to stage changes")?;
    if !status.success() {
        anyhow::bail!("git add failed");
    }

    let status = Command::new("git")
        .args(["commit", "-m", message])
        .status()
        .context("failed to commit")?;
    if !status.success() {
        anyhow::bail!("git commit failed");
    }

    if push {
        let status = Command::new("git")
            .arg("push")
            .status()
            .context("failed to push")?;
        if !status.success() {
            anyhow::bail!("git push failed");
        }
    }
    Ok(())
}

fn git_clone(repo: &str) -> Result<()> {
    let url = if repo.contains("://") || repo.starts_with("git@") {
        repo.to_string()
    } else {
        format!("https://github.com/{repo}.git")
    };
    let status = Command::new("git")
        .args(["clone", &url])
        .status()
        .context("failed to clone")?;
    if !status.success() {
        anyhow::bail!("git clone failed");
    }
    Ok(())
}

fn git_timestamp(message: &str) -> Result<()> {
    let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%z");
    let full_msg = format!("{message} [{ts}]");
    git_commit(&full_msg, false)
}

fn git_clean_branches() -> Result<()> {
    let output = Command::new("git")
        .args(["branch", "--merged"])
        .output()
        .context("failed to list merged branches")?;

    let branches = String::from_utf8_lossy(&output.stdout);
    let mut deleted = 0;

    for line in branches.lines() {
        let branch = line.trim().trim_start_matches("* ");
        // Never delete main, master, develop, or the current branch
        if branch == "main" || branch == "master" || branch == "develop" || line.starts_with('*') {
            continue;
        }
        if branch.is_empty() {
            continue;
        }
        let status = Command::new("git")
            .args(["branch", "-d", branch])
            .status()
            .context(format!("failed to delete branch {branch}"))?;
        if status.success() {
            deleted += 1;
        }
    }

    println!("deleted {deleted} merged branch(es)");
    Ok(())
}

fn git_tree(count: usize) -> Result<()> {
    let status = Command::new("git")
        .args([
            "log",
            "--oneline",
            "--graph",
            "--decorate",
            "--all",
            &format!("-{count}"),
            "--color=always",
        ])
        .status()
        .context("failed to show git tree")?;
    if !status.success() {
        anyhow::bail!("git log failed");
    }
    Ok(())
}
