use std::{net::SocketAddr, sync::Arc};

use clap::Parser;
use kagi_tavily_bridge::{
    http::{app_router, AppState},
    kagi::KagiOpenApiClient,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, env = "BIND_ADDR", default_value = "127.0.0.1:8080")]
    bind_addr: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let kagi = Arc::new(KagiOpenApiClient::new());
    let app = app_router(AppState::new(kagi));

    let listener = tokio::net::TcpListener::bind(args.bind_addr).await?;
    tracing::info!("listening on http://{}", args.bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
