use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct NetArgs {
    #[command(subcommand)]
    command: NetCommand,
}

#[derive(Subcommand)]
enum NetCommand {
    /// Show public IP address
    Ip,
    /// Show local IP address
    Localip,
    /// Ping a host with graphical output
    Ping {
        /// Host to ping
        host: String,
    },
    /// Serve a directory over HTTP
    Serve {
        /// Port number
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Directory to serve
        #[arg(default_value = ".")]
        dir: String,
    },
    /// Kill process listening on a port
    Killport {
        /// Port number
        port: u16,
    },
    /// Show weather for a location
    Weather {
        /// City name (default: auto-detect)
        #[arg(default_value = "")]
        city: String,
    },
}

impl NetArgs {
    pub async fn run(self) -> Result<()> {
        match self.command {
            NetCommand::Ip => public_ip().await,
            NetCommand::Localip => local_ip(),
            NetCommand::Ping { host } => ping(&host),
            NetCommand::Serve { port, dir } => serve(port, &dir),
            NetCommand::Killport { port } => killport(port),
            NetCommand::Weather { city } => weather(&city).await,
        }
    }
}

async fn public_ip() -> Result<()> {
    let resp = reqwest::get("https://ifconfig.me/ip")
        .await
        .context("failed to fetch public IP")?
        .text()
        .await?;
    println!("{}", resp.trim());
    Ok(())
}

fn local_ip() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("ipconfig")
            .args(["getifaddr", "en0"])
            .output()
            .context("failed to get local IP")?;
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("hostname")
            .arg("-I")
            .output()
            .context("failed to get local IP")?;
        let ips = String::from_utf8_lossy(&output.stdout);
        if let Some(ip) = ips.split_whitespace().next() {
            println!("{ip}");
        }
    }
    Ok(())
}

fn ping(host: &str) -> Result<()> {
    // Prefer gping (graphical) if available
    if which::which("gping").is_ok() {
        Command::new("gping")
            .arg(host)
            .status()
            .context("failed to run gping")?;
    } else {
        Command::new("ping")
            .arg("-c")
            .arg("10")
            .arg(host)
            .status()
            .context("failed to run ping")?;
    }
    Ok(())
}

fn serve(port: u16, dir: &str) -> Result<()> {
    if which::which("miniserve").is_ok() {
        Command::new("miniserve")
            .arg("--port")
            .arg(port.to_string())
            .arg(dir)
            .status()
            .context("failed to run miniserve")?;
    } else {
        Command::new("python3")
            .args(["-m", "http.server", &port.to_string()])
            .current_dir(dir)
            .status()
            .context("failed to start HTTP server")?;
    }
    Ok(())
}

fn killport(port: u16) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("lsof")
            .args(["-ti", &format!(":{port}")])
            .output()
            .context("failed to find process on port")?;
        let pids = String::from_utf8_lossy(&output.stdout);
        for pid in pids.lines() {
            let pid = pid.trim();
            if !pid.is_empty() {
                Command::new("kill")
                    .args(["-9", pid])
                    .status()
                    .context(format!("failed to kill pid {pid}"))?;
                println!("killed pid {pid} on port {port}");
            }
        }
        if pids.trim().is_empty() {
            println!("no process found on port {port}");
        }
    }
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("fuser")
            .args(["-k", &format!("{port}/tcp")])
            .output()
            .context("failed to kill process on port")?;
        if output.status.success() {
            println!("killed process on port {port}");
        } else {
            println!("no process found on port {port}");
        }
    }
    Ok(())
}

/// Run weather lookup. Public for multicall dispatch (blx-weather needs async).
pub async fn run_weather(city: &str) -> Result<()> {
    weather(city).await
}

async fn weather(city: &str) -> Result<()> {
    let url = if city.is_empty() {
        "https://wttr.in?format=3".to_string()
    } else {
        format!("https://wttr.in/{city}?format=3")
    };
    let resp = reqwest::get(&url)
        .await
        .context("failed to fetch weather")?
        .text()
        .await?;
    print!("{resp}");
    Ok(())
}
