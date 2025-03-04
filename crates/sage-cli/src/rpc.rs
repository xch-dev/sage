use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use sage::Sage;
use sage_api_macro::impl_endpoints;
use sage_client::{Client, SageRpcError};
use sage_rpc::start_rpc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

impl_endpoints! {
    #[derive(Debug, Parser)]
    #[clap(rename_all = "snake_case")]
    pub enum RpcCommand {
        Start,
        (repeat Endpoint { #[clap(value_parser = parse_with_serde::<sage_api::Endpoint>)] body: sage_api::Endpoint } ,)
    }

    impl RpcCommand {
        pub async fn handle(self, path: PathBuf) -> anyhow::Result<()> {
            match self {
                Self::Start => {
                    let mut sage = Sage::new(&path);
                    let mut receiver = sage.initialize().await?;
                    tokio::spawn(async move { while let Some(_message) = receiver.recv().await {} });
                    start_rpc(Arc::new(Mutex::new(sage))).await
                },
                (repeat Self::Endpoint { body } => {
                    let client = Client::new()?;
                    handle(client.endpoint(body).await);
                    Ok(())
                } ,)
            }
        }
    }
}

fn handle<T: Serialize>(result: Result<T, SageRpcError>) {
    match result {
        Ok(result) => println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("could not serialize result")
        ),
        Err(SageRpcError::Api(_, message)) => eprintln!("{message}"),
        Err(error) => eprintln!("{error}"),
    }
}

fn parse_with_serde<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, String> {
    serde_json::from_str(s).map_err(|error| error.to_string())
}
