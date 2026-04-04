use async_trait::async_trait;
use langchain_rust::{
    embedding::Embedder,
    schemas::Document,
    vectorstore::{VecStoreOptions, VectorStore},
};
use qdrant_client::qdrant::{Filter, PointStruct, SearchPointsBuilder, UpsertPointsBuilder};
use qdrant_client::Qdrant;
use qdrant_client::Payload;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

const COLLECTION_NAME: &str = "knowledge";

/// Qdrant vector store that wraps langchain-rust's VectorStore with company-scoped filtering.
pub struct HivemindVectorStore {
    client: Qdrant,
    embedder: Arc<dyn Embedder>,
    collection_name: String,
}

impl HivemindVectorStore {
    pub async fn new(url: &str, api_key: &str, embedder: Arc<dyn Embedder>) -> anyhow::Result<Self> {
        let mut builder = Qdrant::from_url(url);
        if !api_key.is_empty() {
            builder = builder.api_key(api_key);
        }
        let client = builder.build()?;

        Ok(Self {
            client,
            embedder,
            collection_name: COLLECTION_NAME.to_string(),
        })
    }

    /// Ensure the collection exists, creating it with the embedder's dimension.
    pub async fn ensure_collection(&self) -> anyhow::Result<()> {
        let exists = match self.client.collection_exists(&self.collection_name).await {
            Ok(exists) => exists,
            Err(e) => {
                tracing::warn!(error = %e, "Could not check Qdrant collection existence");
                return Ok(()); // Don't crash on Qdrant errors
            }
        };
        if exists {
            return Ok(());
        }

        // Get embedding dimension by embedding a test string, fallback to default
        let dimension = match self.embedder.embed_query("test").await {
            Ok(embedding) => embedding.len() as u64,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Could not determine embedding dimension from API, using default 1536 (text-embedding-3-small)"
                );
                1536u64
            }
        };

        use qdrant_client::qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder};

        // Try to create the collection, but don't fail if it already exists
        // (another instance may have created it)
        if let Err(e) = self.client
            .create_collection(
                CreateCollectionBuilder::new(&self.collection_name)
                    .vectors_config(VectorParamsBuilder::new(dimension, Distance::Cosine)),
            )
            .await
        {
            tracing::warn!(error = %e, "Could not create Qdrant collection, it may already exist or be unavailable");
        } else {
            tracing::info!(
                collection = %self.collection_name,
                dimension = dimension,
                "Qdrant collection created"
            );
        }
        Ok(())
    }

    /// Build a company-scoped filter for Qdrant queries.
    fn company_filter(company_id: Uuid) -> Filter {
        Filter::must([qdrant_client::qdrant::Condition::matches(
            "company_id",
            company_id.to_string(),
        )])
    }

    /// Add documents to the store with company_id in payload.
    pub async fn add_documents_for_company(
        &self,
        company_id: Uuid,
        docs: &[Document],
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let texts: Vec<String> = docs.iter().map(|d| d.page_content.clone()).collect();
        let vectors = self.embedder.embed_documents(&texts).await?;

        let mut points = Vec::with_capacity(docs.len());
        let mut ids = Vec::with_capacity(docs.len());

        for (doc, vector) in docs.iter().zip(vectors.iter()) {
            let point_id = Uuid::new_v4().to_string();
            ids.push(point_id.clone());

            let vector_f32: Vec<f32> = vector.iter().map(|f| *f as f32).collect();

            let payload = json!({
                "page_content": doc.page_content,
                "metadata": doc.metadata,
                "company_id": company_id.to_string(),
            });

            let point = PointStruct::new(
                point_id,
                vector_f32,
                Payload::try_from(payload).unwrap(),
            );
            points.push(point);
        }

        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection_name, points).wait(true))
            .await?;

        Ok(ids)
    }

    /// Search with company scope.
    pub async fn search_for_company(
        &self,
        company_id: Uuid,
        query: &str,
        limit: usize,
    ) -> Result<Vec<Document>, Box<dyn Error>> {
        let query_vector: Vec<f32> = self
            .embedder
            .embed_query(query)
            .await?
            .into_iter()
            .map(|f| f as f32)
            .collect();

        let operation = SearchPointsBuilder::new(&self.collection_name, query_vector, limit as u64)
            .filter(Self::company_filter(company_id))
            .with_payload(true);

        let results = self.client.search_points(operation).await?;

        let documents = results
            .result
            .into_iter()
            .map(|scored_point| {
                let payload = scored_point.payload;

                let page_content = payload
                    .get("page_content")
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                let metadata: HashMap<String, Value> = payload
                    .get("metadata")
                    .and_then(|v| serde_json::from_value(v.clone().into_json()).ok())
                    .unwrap_or_default();

                let score = scored_point.score as f64;

                Document {
                    page_content,
                    metadata,
                    score,
                }
            })
            .collect();

        Ok(documents)
    }

    /// Delete all points for a meeting.
    pub async fn delete_for_meeting(&self, meeting_id: Uuid) -> anyhow::Result<()> {
        let filter = Filter::must([qdrant_client::qdrant::Condition::matches(
            "metadata.meeting_id",
            meeting_id.to_string(),
        )]);

        self.client
            .delete_points(
                qdrant_client::qdrant::DeletePointsBuilder::new(&self.collection_name)
                    .points(filter)
                    .wait(true),
            )
            .await?;

        Ok(())
    }

    /// Delete all points for a knowledge document.
    pub async fn delete_for_document(&self, document_id: Uuid) -> anyhow::Result<()> {
        let filter = Filter::must([qdrant_client::qdrant::Condition::matches(
            "metadata.document_id",
            document_id.to_string(),
        )]);

        self.client
            .delete_points(
                qdrant_client::qdrant::DeletePointsBuilder::new(&self.collection_name)
                    .points(filter)
                    .wait(true),
            )
            .await?;

        Ok(())
    }
}

#[async_trait]
impl VectorStore for HivemindVectorStore {
    type Options = VecStoreOptions<Value>;

    async fn add_documents(
        &self,
        _docs: &[Document],
        _opt: &VecStoreOptions<Value>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        // This is a fallback; use add_documents_for_company instead for company-scoped inserts
        Err("Use add_documents_for_company with company_id".into())
    }

    async fn similarity_search(
        &self,
        _query: &str,
        _limit: usize,
        _opt: &VecStoreOptions<Value>,
    ) -> Result<Vec<Document>, Box<dyn Error>> {
        // Fallback without company scope
        Err("Use search_for_company with company_id".into())
    }
}
