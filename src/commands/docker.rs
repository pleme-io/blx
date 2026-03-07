use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct DockerArgs {
    #[command(subcommand)]
    command: DockerCommand,
}

#[derive(Subcommand)]
enum DockerCommand {
    /// Remove dangling images, stopped containers, and unused networks
    Clean,
    /// Remove all containers (including running)
    RmAll,
    /// Stop all running containers
    StopAll,
}

impl DockerArgs {
    pub fn run(self) -> Result<()> {
        match self.command {
            DockerCommand::Clean => docker_clean(),
            DockerCommand::RmAll => docker_rm_all(),
            DockerCommand::StopAll => docker_stop_all(),
        }
    }
}

fn docker_clean() -> Result<()> {
    println!("removing stopped containers...");
    Command::new("docker")
        .args(["container", "prune", "-f"])
        .status()
        .context("failed to prune containers")?;

    println!("removing dangling images...");
    Command::new("docker")
        .args(["image", "prune", "-f"])
        .status()
        .context("failed to prune images")?;

    println!("removing unused networks...");
    Command::new("docker")
        .args(["network", "prune", "-f"])
        .status()
        .context("failed to prune networks")?;

    println!("removing unused volumes...");
    Command::new("docker")
        .args(["volume", "prune", "-f"])
        .status()
        .context("failed to prune volumes")?;

    println!("docker cleanup complete");
    Ok(())
}

fn docker_rm_all() -> Result<()> {
    let output = Command::new("docker")
        .args(["ps", "-aq"])
        .output()
        .context("failed to list containers")?;

    let ids = String::from_utf8_lossy(&output.stdout);
    let ids: Vec<&str> = ids.lines().filter(|l| !l.is_empty()).collect();

    if ids.is_empty() {
        println!("no containers to remove");
        return Ok(());
    }

    let mut cmd = Command::new("docker");
    cmd.arg("rm").arg("-f");
    for id in &ids {
        cmd.arg(id);
    }
    cmd.status().context("failed to remove containers")?;
    println!("removed {} container(s)", ids.len());
    Ok(())
}

fn docker_stop_all() -> Result<()> {
    let output = Command::new("docker")
        .args(["ps", "-q"])
        .output()
        .context("failed to list running containers")?;

    let ids = String::from_utf8_lossy(&output.stdout);
    let ids: Vec<&str> = ids.lines().filter(|l| !l.is_empty()).collect();

    if ids.is_empty() {
        println!("no running containers");
        return Ok(());
    }

    let mut cmd = Command::new("docker");
    cmd.arg("stop");
    for id in &ids {
        cmd.arg(id);
    }
    cmd.status().context("failed to stop containers")?;
    println!("stopped {} container(s)", ids.len());
    Ok(())
}
