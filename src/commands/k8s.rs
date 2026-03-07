use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct K8sArgs {
    #[command(subcommand)]
    command: K8sCommand,
}

#[derive(Subcommand)]
enum K8sCommand {
    /// Stream logs from a pod (interactive selection)
    Log {
        /// Pod name (interactive selection if omitted)
        pod: Option<String>,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// Exec into a pod (interactive selection)
    Exec {
        /// Pod name (interactive selection if omitted)
        pod: Option<String>,
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
        /// Command to execute
        #[arg(short, long, default_value = "/bin/bash")]
        command: String,
    },
}

impl K8sArgs {
    pub fn run(self) -> Result<()> {
        match self.command {
            K8sCommand::Log { pod, namespace } => k8s_log(pod.as_deref(), namespace.as_deref()),
            K8sCommand::Exec {
                pod,
                namespace,
                command,
            } => k8s_exec(pod.as_deref(), namespace.as_deref(), &command),
        }
    }
}

fn select_pod(namespace: Option<&str>) -> Result<String> {
    let selector = if which::which("sk").is_ok() { "sk" } else { "fzf" };
    let ns_args = match namespace {
        Some(ns) => format!("-n {ns}"),
        None => String::new(),
    };

    let script = format!(
        "kubectl get pods {ns_args} | {selector} | awk '{{print $1}}'"
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(&script)
        .output()
        .context("failed to select pod")?;

    let pod = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if pod.is_empty() {
        anyhow::bail!("no pod selected");
    }
    Ok(pod)
}

fn k8s_log(pod: Option<&str>, namespace: Option<&str>) -> Result<()> {
    let pod_name = match pod {
        Some(p) => p.to_string(),
        None => select_pod(namespace)?,
    };

    let mut cmd = Command::new("kubectl");
    cmd.arg("logs").arg("-f");
    if let Some(ns) = namespace {
        cmd.arg("-n").arg(ns);
    }
    cmd.arg(&pod_name);
    cmd.status().context("failed to stream logs")?;
    Ok(())
}

fn k8s_exec(pod: Option<&str>, namespace: Option<&str>, command: &str) -> Result<()> {
    let pod_name = match pod {
        Some(p) => p.to_string(),
        None => select_pod(namespace)?,
    };

    let mut cmd = Command::new("kubectl");
    cmd.arg("exec").arg("-it");
    if let Some(ns) = namespace {
        cmd.arg("-n").arg(ns);
    }
    cmd.arg(&pod_name).arg("--").arg(command);
    cmd.status().context("failed to exec into pod")?;
    Ok(())
}
