mod app_state;
mod router;
mod rpc;
mod tls;

use anyhow::Result;
use clap::Parser;
use rpc::start_rpc;
use rustls::crypto::aws_lc_rs::default_provider;

#[derive(Debug, Parser)]
struct Args {
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

#[derive(Debug, Parser)]
enum RpcCommand {
    Start,
}

#[tokio::main]
async fn main() -> Result<()> {
    default_provider()
        .install_default()
        .expect("could not install AWS LC provider");

    let path = dirs::data_dir()
        .expect("could not find data directory")
        .join("com.rigidnetwork.sage");

    let args = Args::parse();

    match args.command {
        Command::Rpc { command } => match command {
            RpcCommand::Start => start_rpc(path).await?,
        },
    }

    Ok(())
}
