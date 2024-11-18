mod app_state;
mod router;

use app_state::AppState;
use router::api_router;
use sage::Sage;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let path = dirs::data_dir()
        .expect("could not find data directory")
        .join("com.rigidnetwork.sage");

    let mut app = Sage::new(&path);
    let mut receiver = app.initialize().await?;

    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            println!("{message:?}");
        }
    });

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    let addr = listener.local_addr()?;
    info!("RPC server is listening at {addr}");

    let app = api_router().with_state(AppState {});
    axum::serve(listener, app).await?;

    Ok(())
}
