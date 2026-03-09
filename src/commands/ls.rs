//! `blx ls` — POSIX-compatible ls wrapper that delegates to eza.
//!
//! Translates standard flags (-l, -a, -t, -r, -1) to eza equivalents
//! while preserving icons and group-directories-first defaults.

use anyhow::{Context, Result};
use std::process::Command;

/// Run ls with the given raw arguments, translating POSIX flags to eza.
pub fn run(args: &[String]) -> Result<()> {
    let mut eza_args = vec![
        "--icons".to_string(),
        "--group-directories-first".to_string(),
    ];
    let mut paths = Vec::new();

    for arg in args {
        if arg.starts_with("--") {
            eza_args.push(arg.clone());
        } else if arg.starts_with('-') {
            if arg.contains('l') {
                eza_args.push("-l".into());
            }
            if arg.contains('a') || arg.contains('A') {
                eza_args.push("-a".into());
            }
            if arg.contains('t') {
                eza_args.push("--sort=modified".into());
            }
            if arg.contains('r') {
                eza_args.push("--reverse".into());
            }
            if arg.contains('1') {
                eza_args.push("-1".into());
            }
            if arg.contains('R') {
                eza_args.push("--recurse".into());
            }
            if arg.contains('S') {
                eza_args.push("--sort=size".into());
            }
            if arg.contains('h') {
                // eza always uses human-readable sizes in -l mode
            }
        } else {
            paths.push(arg.clone());
        }
    }

    eza_args.extend(paths);

    let status = Command::new("eza")
        .args(&eza_args)
        .status()
        .context("failed to run eza")?;

    std::process::exit(status.code().unwrap_or(1));
}
