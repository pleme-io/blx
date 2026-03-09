mod commands;
mod config;

use clap::Parser;
use commands::Cli;

/// Multicall dispatch: when invoked via `blx-*` symlink, route directly
/// to the appropriate Rust function without going through clap.
fn try_multicall() -> Option<anyhow::Result<()>> {
    let argv0 = std::env::args().next()?;
    let bin_name = std::path::Path::new(&argv0).file_name()?.to_str()?;
    let suffix = bin_name.strip_prefix("blx-")?;
    let args: Vec<String> = std::env::args().skip(1).collect();

    let result = match suffix {
        "ls" => commands::ls::run(&args),

        "preview" => {
            let path = args.first().map(String::as_str).unwrap_or(".");
            commands::preview::preview_file(path)
        }
        "preview-dir" => {
            let path = args.first().map(String::as_str).unwrap_or(".");
            commands::preview::preview_dir(path)
        }
        "preview-proc" => {
            let group = args.first().map(String::as_str).unwrap_or("");
            let word = args.get(1).map(String::as_str).unwrap_or("");
            commands::preview::preview_proc(group, word)
        }
        "preview-git" => {
            let context = args.first().map(String::as_str).unwrap_or("log");
            let word = args.get(1).map(String::as_str).unwrap_or("");
            let group = args.get(2).map(String::as_str);
            commands::preview::preview_git(context, word, group)
        }

        "backup" => {
            let path = args.first().map(String::as_str).unwrap_or_else(|| {
                eprintln!("usage: blx-backup <file>");
                std::process::exit(1);
            });
            commands::file::backup_file(std::path::Path::new(path))
        }
        "json" => commands::encode::pretty_json_stdin(),
        "urlencode" => {
            let input = args.first().map(String::as_str).unwrap_or_else(|| {
                eprintln!("usage: blx-urlencode <string>");
                std::process::exit(1);
            });
            println!("{}", urlencoding::encode(input));
            Ok(())
        }
        "urldecode" => {
            let input = args.first().map(String::as_str).unwrap_or_else(|| {
                eprintln!("usage: blx-urldecode <string>");
                std::process::exit(1);
            });
            match urlencoding::decode(input) {
                Ok(decoded) => { println!("{decoded}"); Ok(()) }
                Err(e) => { eprintln!("error: {e}"); std::process::exit(1); }
            }
        }
        "weather" => {
            // Weather needs async — rewrite args and fall through to clap
            return None;
        }

        _ => return None,
    };

    Some(result)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Check for blx-* symlink multicall dispatch
    if let Some(result) = try_multicall() {
        return result;
    }

    // Handle blx-weather specially (needs async runtime already running)
    let argv0 = std::env::args().next().unwrap_or_default();
    let bin_name = std::path::Path::new(&argv0)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");
    if bin_name == "blx-weather" {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let city = args.first().map(String::as_str).unwrap_or("");
        return commands::net::run_weather(city).await;
    }

    let cli = Cli::parse();
    cli.run().await
}
