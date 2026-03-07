pub mod docker;
pub mod encode;
pub mod file;
pub mod find;
pub mod git;
pub mod k8s;
pub mod net;
pub mod nix_cmd;
pub mod util;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "blx",
    about = "Blackmatter shell extensions — Rust CLI replacing shell functions",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
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
}

impl Cli {
    pub async fn run(self) -> anyhow::Result<()> {
        match self.command {
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
        }
    }
}
