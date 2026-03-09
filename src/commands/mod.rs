pub mod docker;
pub mod encode;
pub mod file;
pub mod find;
pub mod git;
pub mod init;
pub mod k8s;
pub mod ls;
pub mod net;
pub mod nix_cmd;
pub mod preview;
pub mod util;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "blx",
    about = "Blackmatter shell extensions — config-driven zsh generation + Rust CLI utilities",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate shell configuration (eval "$(blx init zsh)")
    Init(init::InitArgs),
    /// File operations: extract, compress, backup, dirsize
    File(file::FileArgs),
    /// Find files, directories, content, processes
    Find(find::FindArgs),
    /// Git shortcuts: commit, clone, clean-branches, tree
    Git(git::GitArgs),
    /// Network utilities: ip, serve, killport, weather
    Net(net::NetArgs),
    /// Encode data: base64, url, json
    Encode(encode::EncodeArgs),
    /// Decode data: base64, url
    Decode(encode::DecodeArgs),
    /// General utilities: genpass, calc, bench, histstat, mkcd
    Util(util::UtilArgs),
    /// Docker cleanup: clean, rm-all, stop-all
    Docker(docker::DockerArgs),
    /// Kubernetes helpers: log, exec
    K8s(k8s::K8sArgs),
    /// Nix helpers: info, shell-pkg
    Nix(nix_cmd::NixArgs),
    /// POSIX-compatible ls via eza (translates -ltra flags)
    Ls {
        /// Arguments to pass through (POSIX flags translated to eza)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Preview helpers for fzf-tab (file, dir, proc, git)
    Preview(preview::PreviewArgs),
}

impl Cli {
    pub async fn run(self) -> anyhow::Result<()> {
        match self.command {
            Command::Init(args) => args.run(),
            Command::File(args) => args.run(),
            Command::Find(args) => args.run(),
            Command::Git(args) => args.run(),
            Command::Net(args) => args.run().await,
            Command::Encode(args) => args.run(),
            Command::Decode(args) => args.run(),
            Command::Util(args) => args.run(),
            Command::Docker(args) => args.run(),
            Command::K8s(args) => args.run(),
            Command::Nix(args) => args.run(),
            Command::Ls { args } => ls::run(&args),
            Command::Preview(args) => args.run(),
        }
    }
}
