mod rpc;
mod wallet;

use std::thread;

use anyhow::Result;
use clap::Parser;
use rpc::RpcCommand;
use rustls::crypto::aws_lc_rs::default_provider;
use wallet::WalletArgs;

/// The `rpc` command has a subcommand and a match arm for every endpoint. Building it, and
/// the future that handles it, overflows the 1 MiB main-thread stack of a Windows debug build
/// (even `sage --help` did), so the CLI runs on a thread with room to spare.
const STACK_SIZE: usize = 16 * 1024 * 1024;

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Call any RPC endpoint with a JSON body
    Rpc {
        #[clap(subcommand)]
        command: RpcCommand,
    },
    /// Everyday wallet commands with readable input and output
    Wallet(WalletArgs),
}

fn main() -> Result<()> {
    thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)?
        .join()
        .expect("the CLI thread panicked")
}

fn run() -> Result<()> {
    default_provider()
        .install_default()
        .expect("could not install AWS LC provider");

    let args = Args::parse();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(STACK_SIZE)
        .build()?
        .block_on(async move {
            match args.command {
                Command::Rpc { command } => command.handle().await,
                Command::Wallet(args) => args.handle().await,
            }
        })
}
