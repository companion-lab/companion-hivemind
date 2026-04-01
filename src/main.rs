mod config;
mod db;
mod api;

use sqlx::PgPool;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub settings: config::Settings,
}

mod services {
    pub mod auth;
    pub mod crypto;
    pub mod knowledge;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("companion_hivemind=info".parse()?),
        )
        .init();

    let settings = config::load();
    let db = db::connect(&settings).await?;

    let state = AppState { db, settings: settings.clone() };
    let app = api::router(state);

    let addr: SocketAddr = format!("{}:{}", settings.host, settings.port).parse()?;
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
