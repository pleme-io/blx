use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use rand::Rng;
use std::process::Command;

#[derive(Args)]
pub struct UtilArgs {
    #[command(subcommand)]
    command: UtilCommand,
}

#[derive(Subcommand)]
enum UtilCommand {
    /// Generate a random password
    Genpass {
        /// Password length
        #[arg(default_value = "32")]
        length: usize,
    },
    /// Evaluate a math expression
    Calc {
        /// Expression to evaluate
        expression: String,
    },
    /// Benchmark a command using hyperfine
    Bench {
        /// Command to benchmark
        command: String,
        /// Number of runs
        #[arg(short, long, default_value = "10")]
        runs: usize,
    },
    /// Show shell history statistics
    Histstat {
        /// Number of top commands to show
        #[arg(short, long, default_value = "20")]
        count: usize,
    },
    /// Create a directory and print cd command for eval
    Mkcd {
        /// Directory to create
        dir: String,
    },
}

impl UtilArgs {
    pub fn run(self) -> Result<()> {
        match self.command {
            UtilCommand::Genpass { length } => genpass(length),
            UtilCommand::Calc { expression } => calc(&expression),
            UtilCommand::Bench { command, runs } => bench(&command, runs),
            UtilCommand::Histstat { count } => histstat(count),
            UtilCommand::Mkcd { dir } => mkcd(&dir),
        }
    }
}

fn genpass(length: usize) -> Result<()> {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+";
    let mut rng = rand::rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    println!("{password}");
    Ok(())
}

fn calc(expression: &str) -> Result<()> {
    // Use bc for calculation
    let output = Command::new("bc")
        .arg("-l")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(expression.as_bytes())?;
                stdin.write_all(b"\n")?;
            }
            child.wait_with_output()
        })
        .context("failed to run bc")?;

    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

fn bench(command: &str, runs: usize) -> Result<()> {
    if which::which("hyperfine").is_ok() {
        Command::new("hyperfine")
            .arg("--runs")
            .arg(runs.to_string())
            .arg(command)
            .status()
            .context("failed to run hyperfine")?;
    } else {
        eprintln!("hyperfine not found, install it for benchmarking");
    }
    Ok(())
}

fn histstat(count: usize) -> Result<()> {
    // Read zsh history and compute statistics
    let hist_file = std::env::var("HISTFILE").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        let xdg_state = std::env::var("XDG_STATE_HOME")
            .unwrap_or_else(|_| format!("{home}/.local/state"));
        format!("{xdg_state}/zsh/history")
    });

    let contents = match std::fs::read_to_string(&hist_file) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("could not read history file: {hist_file}");
            return Ok(());
        }
    };

    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut total = 0usize;

    for line in contents.lines() {
        // zsh extended history format: : timestamp:0;command
        let cmd = if let Some(idx) = line.find(';') {
            &line[idx + 1..]
        } else {
            line
        };
        let first_word = cmd.split_whitespace().next().unwrap_or("").to_string();
        if !first_word.is_empty() {
            *counts.entry(first_word).or_default() += 1;
            total += 1;
        }
    }

    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{:<6} {:<8} {}", "rank", "count", "command");
    println!("{:-<6} {:-<8} {:-<30}", "", "", "");
    for (i, (cmd, cnt)) in sorted.iter().take(count).enumerate() {
        let pct = (*cnt as f64 / total as f64) * 100.0;
        println!("{:<6} {:<8} {} ({pct:.1}%)", i + 1, cnt, cmd);
    }
    println!("\ntotal commands: {total}");
    Ok(())
}

fn mkcd(dir: &str) -> Result<()> {
    std::fs::create_dir_all(dir).context(format!("failed to create directory: {dir}"))?;
    // Output cd command for shell eval: eval $(blx util mkcd foo)
    println!("cd {dir}");
    Ok(())
}
