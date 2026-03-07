use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct NixArgs {
    #[command(subcommand)]
    command: NixCommand,
}

#[derive(Subcommand)]
enum NixCommand {
    /// Show Nix installation info
    Info,
    /// Quick nix-shell with a package
    Shell {
        /// Package name
        package: String,
    },
}

impl NixArgs {
    pub fn run(self) -> Result<()> {
        match self.command {
            NixCommand::Info => nix_info(),
            NixCommand::Shell { package } => nix_shell(&package),
        }
    }
}

fn nix_info() -> Result<()> {
    println!("=== Nix Version ===");
    Command::new("nix")
        .arg("--version")
        .status()
        .context("failed to get nix version")?;

    println!("\n=== Nix Channels ===");
    let _ = Command::new("nix-channel").arg("--list").status();

    println!("\n=== Store Path ===");
    let output = Command::new("nix")
        .args(["store", "ping", "--json"])
        .output();
    if let Ok(out) = output {
        print!("{}", String::from_utf8_lossy(&out.stdout));
    }

    println!("\n=== Nix Config ===");
    let _ = Command::new("nix").args(["show-config"]).status();

    Ok(())
}

fn nix_shell(package: &str) -> Result<()> {
    let status = Command::new("nix-shell")
        .arg("-p")
        .arg(package)
        .arg("--run")
        .arg("zsh")
        .status()
        .context("failed to start nix-shell")?;
    if !status.success() {
        anyhow::bail!("nix-shell exited with status {status}");
    }
    Ok(())
}
