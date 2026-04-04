use async_trait::async_trait;
use langchain_rust::embedding::{embedder_trait::Embedder, EmbedderError, ollama::OllamaEmbedder as LcOllamaEmbedder};
use ollama_rs::Ollama;
use std::sync::Arc;
use url::Url;

/// Embedding backend supported by the hivemind service.
#[derive(Clone)]
pub enum EmbedderBackend {
    Ollama(Arc<LcOllamaEmbedder>),
}

/// Wrapper that dispatches to the configured embedder backend.
#[derive(Clone)]
pub struct HivemindEmbedder {
    backend: EmbedderBackend,
}

impl HivemindEmbedder {
    /// Create an Ollama-based embedder.
    pub fn new_ollama(base_url: &str, model: &str) -> Self {
        let url = Url::parse(base_url).unwrap_or_else(|_| {
            Url::parse(&format!("http://{}", base_url)).expect("invalid Ollama URL")
        });
        let host = url.host_str().unwrap_or("localhost");
        let port = url.port().unwrap_or(11434);
        let full_url = format!("http://{}:{}", host, port);
        let client = Ollama::try_new(full_url).expect("failed to create Ollama client");
        let embedder = LcOllamaEmbedder::new(Arc::new(client), model, None);
        Self {
            backend: EmbedderBackend::Ollama(Arc::new(embedder)),
        }
    }
}

#[async_trait]
impl Embedder for HivemindEmbedder {
    async fn embed_documents(&self, documents: &[String]) -> Result<Vec<Vec<f64>>, EmbedderError> {
        match &self.backend {
            EmbedderBackend::Ollama(e) => e.embed_documents(documents).await,
        }
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f64>, EmbedderError> {
        match &self.backend {
            EmbedderBackend::Ollama(e) => e.embed_query(text).await,
        }
    }
}
