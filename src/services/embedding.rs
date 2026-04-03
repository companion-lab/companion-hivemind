use async_trait::async_trait;
use langchain_rust::embedding::{embedder_trait::Embedder, EmbedderError, openrouter::OpenrouterEmbedder};
use std::sync::Arc;

/// Wrapper around langchain-rust-openrouter's OpenrouterEmbedder.
#[derive(Clone)]
pub struct HivemindEmbedder {
    inner: Arc<OpenrouterEmbedder>,
}

impl HivemindEmbedder {
    pub fn new(api_key: &str, model: &str, base_url: Option<&str>) -> Self {
        let mut embedder = OpenrouterEmbedder::new(api_key, model);
        if let Some(url) = base_url {
            embedder = embedder.with_base_url(url);
        }
        Self {
            inner: Arc::new(embedder),
        }
    }
}

#[async_trait]
impl Embedder for HivemindEmbedder {
    async fn embed_documents(&self, documents: &[String]) -> Result<Vec<Vec<f64>>, EmbedderError> {
        self.inner.embed_documents(documents).await
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f64>, EmbedderError> {
        self.inner.embed_query(text).await
    }
}
