use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::{Args, Subcommand};
use std::io::{self, Read};

#[derive(Args)]
pub struct EncodeArgs {
    #[command(subcommand)]
    command: EncodeCommand,
}

#[derive(Subcommand)]
enum EncodeCommand {
    /// Encode to base64
    B64 {
        /// Input string (reads stdin if omitted)
        input: Option<String>,
    },
    /// URL-encode a string
    Url {
        /// Input string
        input: String,
    },
    /// Pretty-print JSON (reads stdin)
    Json,
}

impl EncodeArgs {
    pub fn run(self) -> Result<()> {
        match self.command {
            EncodeCommand::B64 { input } => {
                let data = get_input(input)?;
                println!("{}", STANDARD.encode(data.as_bytes()));
                Ok(())
            }
            EncodeCommand::Url { input } => {
                println!("{}", urlencoding::encode(&input));
                Ok(())
            }
            EncodeCommand::Json => pretty_json_stdin(),
        }
    }
}

#[derive(Args)]
pub struct DecodeArgs {
    #[command(subcommand)]
    command: DecodeCommand,
}

#[derive(Subcommand)]
enum DecodeCommand {
    /// Decode from base64
    B64 {
        /// Base64-encoded input (reads stdin if omitted)
        input: Option<String>,
    },
    /// URL-decode a string
    Url {
        /// URL-encoded input
        input: String,
    },
}

impl DecodeArgs {
    pub fn run(self) -> Result<()> {
        match self.command {
            DecodeCommand::B64 { input } => {
                let data = get_input(input)?;
                let decoded = STANDARD
                    .decode(data.trim().as_bytes())
                    .context("invalid base64")?;
                let text = String::from_utf8(decoded).context("decoded data is not valid UTF-8")?;
                println!("{text}");
                Ok(())
            }
            DecodeCommand::Url { input } => {
                println!(
                    "{}",
                    urlencoding::decode(&input).context("invalid URL encoding")?
                );
                Ok(())
            }
        }
    }
}

/// Pretty-print JSON from stdin. Public for multicall dispatch.
pub fn pretty_json_stdin() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).context("failed to read stdin")?;
    let value: serde_json::Value = serde_json::from_str(&input).context("invalid JSON")?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn get_input(arg: Option<String>) -> Result<String> {
    match arg {
        Some(s) => Ok(s),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).context("failed to read stdin")?;
            Ok(buf)
        }
    }
}
