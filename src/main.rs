mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod repos;
mod router;
mod services;
mod types;
mod util;

use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::services::embedding::HivemindEmbedder;
use crate::services::vector::HivemindVectorStore;

/// Shared application state, injected into all handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub settings: config::Settings,
    pub vector_store: Arc<HivemindVectorStore>,
    pub services: AppServices,
}

/// Domain services, initialized once at startup.
#[derive(Clone)]
pub struct AppServices {
    pub auth: services::auth::AuthService,
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
    tracing::info!("Embedding config: url={} model={}", settings.embedding_api_url, settings.embedding_model);
    let db = db::connect(&settings).await?;

    // Initialize embedding service (Ollama)
    let embedder = Arc::new(HivemindEmbedder::new_ollama(
        &settings.embedding_api_url,
        &settings.embedding_model,
    ));

    // Initialize Qdrant vector store
    let vector_store = Arc::new(
        HivemindVectorStore::new(
            &settings.qdrant_url,
            &settings.qdrant_api_key,
            embedder,
        )
        .await?,
    );
    vector_store.ensure_collection().await?;

    let services = AppServices {
        auth: services::auth::AuthService,
    };

    let state = AppState {
        db,
        settings: settings.clone(),
        vector_store,
        services,
    };

    let app = router::build(state);

    let addr: SocketAddr = format!("{}:{}", settings.host, settings.port).parse()?;
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
