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
    /// Strip trailer lines (e.g. Co-Authored-By) from all commits in history
    StripTrailers {
        /// Trailer prefix to strip (case-insensitive match). Default: "Co-Authored-By"
        #[arg(short, long, default_value = "Co-Authored-By")]
        trailer: String,

        /// Only strip trailers matching this value pattern (regex)
        #[arg(short, long)]
        pattern: Option<String>,

        /// Dry run — show what would be stripped without rewriting
        #[arg(long)]
        dry_run: bool,

        /// Force push to origin after rewriting
        #[arg(long)]
        push: bool,

        /// Repository path (defaults to current directory)
        #[arg(short, long)]
        repo: Option<String>,
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
            GitCommand::StripTrailers {
                trailer,
                pattern,
                dry_run,
                push,
                repo,
            } => git_strip_trailers(&trailer, pattern.as_deref(), dry_run, push, repo.as_deref()),
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

fn git_strip_trailers(
    trailer: &str,
    pattern: Option<&str>,
    dry_run: bool,
    push: bool,
    repo: Option<&str>,
) -> Result<()> {
    let work_dir = repo
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine cwd"));

    if !work_dir.join(".git").exists() {
        anyhow::bail!("{} is not a git repository", work_dir.display());
    }

    // Count matching commits first
    let output = Command::new("git")
        .args(["log", "--all", "--format=%H%n%B%n---END---"])
        .current_dir(&work_dir)
        .output()
        .context("failed to read git log")?;

    let log_text = String::from_utf8_lossy(&output.stdout);
    let trailer_lower = trailer.to_lowercase();

    let mut matching_commits: Vec<(String, Vec<String>)> = Vec::new();
    let mut current_hash = String::new();
    let mut current_lines: Vec<String> = Vec::new();

    for line in log_text.lines() {
        if line == "---END---" {
            let matched: Vec<String> = current_lines
                .iter()
                .filter(|l| {
                    let l_lower = l.to_lowercase();
                    if !l_lower.starts_with(&format!("{}:", trailer_lower)) {
                        return false;
                    }
                    match pattern {
                        Some(pat) => l_lower.contains(&pat.to_lowercase()),
                        None => true,
                    }
                })
                .cloned()
                .collect();

            if !matched.is_empty() {
                matching_commits.push((current_hash.clone(), matched));
            }
            current_hash.clear();
            current_lines.clear();
        } else if current_hash.is_empty() && line.len() == 40 && line.chars().all(|c| c.is_ascii_hexdigit()) {
            current_hash = line.to_string();
        } else {
            current_lines.push(line.to_string());
        }
    }

    if matching_commits.is_empty() {
        println!("no commits found with trailer '{trailer}'");
        return Ok(());
    }

    println!(
        "found {} commit(s) with '{}' trailers{}",
        matching_commits.len(),
        trailer,
        pattern.map_or(String::new(), |p| format!(" matching '{p}'"))
    );

    for (hash, lines) in &matching_commits {
        let short = &hash[..8];
        for line in lines {
            println!("  {short}: {line}");
        }
    }

    if dry_run {
        println!("\ndry run — no changes made");
        return Ok(());
    }

    // Build sed filter expression
    // macOS sed doesn't support case-insensitive flag, so we match the exact trailer
    let sed_pattern = match pattern {
        Some(pat) => format!("/^{trailer}:.*{pat}/d"),
        None => format!("/^{trailer}:/d"),
    };

    println!("\nrewriting history...");

    let status = Command::new("git")
        .args(["filter-branch", "-f", "--msg-filter", &format!("sed '{sed_pattern}'"), "--", "--all"])
        .current_dir(&work_dir)
        .env("FILTER_BRANCH_SQUELCH_WARNING", "1")
        .status()
        .context("git filter-branch failed")?;

    if !status.success() {
        anyhow::bail!("git filter-branch failed");
    }

    // Clean up filter-branch backup refs
    let _ = Command::new("bash")
        .args(["-c", "rm -rf .git/refs/original/"])
        .current_dir(&work_dir)
        .status();

    // Run gc to drop unreachable objects
    let _ = Command::new("git")
        .args(["reflog", "expire", "--expire=now", "--all"])
        .current_dir(&work_dir)
        .status();
    let _ = Command::new("git")
        .args(["gc", "--prune=now"])
        .current_dir(&work_dir)
        .status();

    // Verify on current branch (not --all which includes stale remote refs)
    let verify = Command::new("git")
        .args(["log", "--format=%b"])
        .current_dir(&work_dir)
        .output()
        .context("failed to verify")?;

    let remaining = String::from_utf8_lossy(&verify.stdout)
        .lines()
        .filter(|l| {
            let l_lower = l.to_lowercase();
            if !l_lower.starts_with(&format!("{}:", trailer_lower)) {
                return false;
            }
            match pattern {
                Some(pat) => l_lower.contains(&pat.to_lowercase()),
                None => true,
            }
        })
        .count();

    println!("done. remaining matching trailers: {remaining}");

    if push {
        let branch = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&work_dir)
            .output()
            .context("failed to get branch")?;
        let branch_name = String::from_utf8_lossy(&branch.stdout).trim().to_string();

        println!("force-pushing {branch_name} to origin...");
        let status = Command::new("git")
            .args(["push", "--force", "origin", &branch_name])
            .current_dir(&work_dir)
            .status()
            .context("failed to push")?;

        if !status.success() {
            anyhow::bail!("git push failed");
        }
        println!("pushed.");
    }

    Ok(())
}
