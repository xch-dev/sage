mod rpc;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use rpc::RpcCommand;
use rustls::crypto::aws_lc_rs::default_provider;

#[derive(Debug, Parser)]
struct Args {
    /// Override the data directory
    #[clap(long, global = true)]
    data_dir: Option<PathBuf>,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Rpc {
        #[clap(subcommand)]
        command: RpcCommand,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    default_provider()
        .install_default()
        .expect("could not install AWS LC provider");

    let args = Args::parse();

    let path = if let Some(dir) = args.data_dir {
        dir
    } else {
        dirs::data_dir()
            .expect("could not get data directory")
            .join("com.rigidnetwork.sage")
    };

    match args.command {
        Command::Rpc { command } => command.handle(path).await?,
    }

    Ok(())
}
