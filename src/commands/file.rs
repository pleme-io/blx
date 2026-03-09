use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Args)]
pub struct FileArgs {
    #[command(subcommand)]
    command: FileCommand,
}

#[derive(Subcommand)]
enum FileCommand {
    /// Extract an archive (tar, gz, bz2, xz, zip, 7z, rar, zst)
    Extract {
        /// Path to archive file
        archive: PathBuf,
        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Compress a file or directory
    Compress {
        /// Path to file or directory
        source: PathBuf,
        /// Output archive path
        output: PathBuf,
    },
    /// Create a timestamped backup of a file
    Backup {
        /// File to back up
        file: PathBuf,
    },
    /// Show directory sizes (sorted)
    Dirsize {
        /// Directory to analyze (default: current)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

impl FileArgs {
    pub fn run(self) -> Result<()> {
        match self.command {
            FileCommand::Extract { archive, output } => extract(&archive, output.as_deref()),
            FileCommand::Compress { source, output } => compress(&source, &output),
            FileCommand::Backup { file } => backup_file(&file),
            FileCommand::Dirsize { path } => dirsize(&path),
        }
    }
}

fn extract(archive: &Path, output: Option<&Path>) -> Result<()> {
    if !archive.exists() {
        bail!("archive not found: {}", archive.display());
    }

    // Prefer ouch if available (Rust, handles everything)
    if which::which("ouch").is_ok() {
        let mut cmd = Command::new("ouch");
        cmd.arg("decompress").arg(archive);
        if let Some(out) = output {
            cmd.arg("--dir").arg(out);
        }
        let status = cmd.status().context("failed to run ouch")?;
        if !status.success() {
            bail!("ouch exited with status {}", status);
        }
        return Ok(());
    }

    // Fallback: detect format and use standard tools
    let name = archive.to_string_lossy();
    let (cmd_name, args): (&str, Vec<&str>) = if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        ("tar", vec!["xzf"])
    } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
        ("tar", vec!["xjf"])
    } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        ("tar", vec!["xJf"])
    } else if name.ends_with(".tar.zst") {
        ("tar", vec!["--zstd", "-xf"])
    } else if name.ends_with(".tar") {
        ("tar", vec!["xf"])
    } else if name.ends_with(".gz") {
        ("gunzip", vec![])
    } else if name.ends_with(".bz2") {
        ("bunzip2", vec![])
    } else if name.ends_with(".xz") {
        ("unxz", vec![])
    } else if name.ends_with(".zip") {
        ("unzip", vec![])
    } else if name.ends_with(".7z") {
        ("7z", vec!["x"])
    } else if name.ends_with(".rar") {
        ("unrar", vec!["x"])
    } else if name.ends_with(".zst") {
        ("unzstd", vec![])
    } else {
        bail!("unknown archive format: {}", name);
    };

    let mut cmd = Command::new(cmd_name);
    for a in &args {
        cmd.arg(a);
    }
    cmd.arg(archive);
    if let Some(out) = output {
        if cmd_name == "tar" {
            cmd.arg("-C").arg(out);
        } else if cmd_name == "unzip" {
            cmd.arg("-d").arg(out);
        }
    }

    let status = cmd.status().context(format!("failed to run {cmd_name}"))?;
    if !status.success() {
        bail!("{cmd_name} exited with status {status}");
    }
    Ok(())
}

fn compress(source: &Path, output: &Path) -> Result<()> {
    if !source.exists() {
        bail!("source not found: {}", source.display());
    }

    // Prefer ouch if available
    if which::which("ouch").is_ok() {
        let status = Command::new("ouch")
            .arg("compress")
            .arg(source)
            .arg(output)
            .status()
            .context("failed to run ouch")?;
        if !status.success() {
            bail!("ouch exited with status {}", status);
        }
        return Ok(());
    }

    // Fallback: detect from output extension
    let name = output.to_string_lossy();
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        let status = Command::new("tar")
            .arg("czf")
            .arg(output)
            .arg(source)
            .status()
            .context("failed to run tar")?;
        if !status.success() {
            bail!("tar exited with status {status}");
        }
    } else if name.ends_with(".zip") {
        let status = Command::new("zip")
            .arg("-r")
            .arg(output)
            .arg(source)
            .status()
            .context("failed to run zip")?;
        if !status.success() {
            bail!("zip exited with status {status}");
        }
    } else {
        bail!("unsupported output format: {name}. Use ouch for full format support.");
    }
    Ok(())
}

/// Create a timestamped backup of a file. Public for multicall dispatch.
pub fn backup_file(file: &Path) -> Result<()> {
    if !file.exists() {
        bail!("file not found: {}", file.display());
    }
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}.{timestamp}.bak", file.display());
    std::fs::copy(file, &backup_name).context("failed to copy file")?;
    println!("backed up to {backup_name}");
    Ok(())
}

fn dirsize(path: &Path) -> Result<()> {
    // Prefer dust if available
    if which::which("dust").is_ok() {
        let status = Command::new("dust")
            .arg(path)
            .status()
            .context("failed to run dust")?;
        if !status.success() {
            bail!("dust exited with status {status}");
        }
        return Ok(());
    }

    let status = Command::new("du")
        .arg("-sh")
        .arg(path)
        .status()
        .context("failed to run du")?;
    if !status.success() {
        bail!("du exited with status {status}");
    }
    Ok(())
}
