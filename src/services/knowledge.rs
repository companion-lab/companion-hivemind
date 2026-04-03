use uuid::Uuid;
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use langchain_rust::schemas::Document;

use crate::errors::AppError;
use crate::types::{KnowledgeSearchResult, TranscriptSegment};
use crate::services::vector::HivemindVectorStore;

const CHUNK_SIZE: usize = 400;
const CHUNK_OVERLAP: usize = 80;

pub struct KnowledgeService;

/// A knowledge document ready for vector ingestion.
pub struct KnowledgeDocument {
    pub content: String,
    pub metadata: serde_json::Value,
}

impl KnowledgeService {
    /// Chunk a transcript into langchain-rust Documents with metadata.
    pub fn chunk_transcript(
        segments: &[TranscriptSegment],
        meeting_id: Uuid,
        meeting_date: i64,
    ) -> Vec<Document> {
        let mut docs = Vec::new();

        for seg in segments {
            let sub_chunks = split_text(&seg.text, CHUNK_SIZE, CHUNK_OVERLAP);
            for text in sub_chunks {
                let mut metadata: HashMap<String, serde_json::Value> = HashMap::new();
                metadata.insert("meeting_id".to_string(), serde_json::json!(meeting_id.to_string()));
                metadata.insert("start".to_string(), serde_json::json!(seg.start_time));
                metadata.insert("end".to_string(), serde_json::json!(seg.end_time));
                metadata.insert("chunk_type".to_string(), serde_json::json!("transcript"));

                if !seg.speaker.is_empty() {
                    metadata.insert("speaker".to_string(), serde_json::json!(seg.speaker));
                }
                metadata.insert(
                    "timestamp".to_string(),
                    serde_json::json!(meeting_date + (seg.start_time * 1000.0) as i64),
                );

                docs.push(Document {
                    page_content: text,
                    metadata,
                    score: 0.0,
                });
            }
        }
        docs
    }

    /// Ingest chunked documents into both Postgres (for persistence) and Qdrant (for vector search).
    pub async fn ingest_documents(
        db: &PgPool,
        vector_store: &HivemindVectorStore,
        company_id: Uuid,
        meeting_id: Uuid,
        docs: &[Document],
    ) -> Result<(), AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before UNIX epoch")
            .as_millis() as i64;

        for doc in docs {
            let speaker = doc.metadata.get("speaker").and_then(|v| v.as_str()).map(String::from);
            let timestamp = doc.metadata.get("timestamp").and_then(|v| v.as_i64());
            let metadata_json = serde_json::to_value(&doc.metadata).unwrap_or(serde_json::json!({}));

            sqlx::query(
                "INSERT INTO knowledge_chunks (id, meeting_id, text, speaker, timestamp, chunk_type, metadata, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(Uuid::new_v4())
            .bind(meeting_id)
            .bind(&doc.page_content)
            .bind(&speaker)
            .bind(timestamp)
            .bind("transcript")
            .bind(&metadata_json)
            .bind(now)
            .execute(db)
            .await
            .map_err(AppError::from)?;
        }

        vector_store
            .add_documents_for_company(company_id, docs)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to add documents to vector store: {}", e)))?;

        Ok(())
    }

    /// Search knowledge using vector similarity via Qdrant.
    pub async fn search(
        db: &PgPool,
        vector_store: &HivemindVectorStore,
        company_id: Uuid,
        query: &str,
        limit: usize,
    ) -> Result<Vec<KnowledgeSearchResult>, AppError> {
        let docs = vector_store
            .search_for_company(company_id, query, limit)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Vector search failed: {}", e)))?;

        let mut results = Vec::new();

        for doc in docs {
            let meeting_id = doc
                .metadata
                .get("meeting_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());

            if let Some(mid) = meeting_id {
                let meeting_row = sqlx::query(
                    "SELECT id, title, date FROM meetings WHERE id = $1 AND company_id = $2",
                )
                .bind(mid)
                .bind(company_id)
                .fetch_optional(db)
                .await
                .ok()
                .flatten();

                let chunk_json = serde_json::json!({
                    "text": doc.page_content,
                    "speaker": doc.metadata.get("speaker").and_then(|v| v.as_str()),
                    "timestamp": doc.metadata.get("timestamp").and_then(|v| v.as_i64()),
                    "chunk_type": doc.metadata.get("chunk_type").and_then(|v| v.as_str()).unwrap_or("transcript"),
                    "metadata": doc.metadata,
                    "meeting_id": mid.to_string(),
                });

                let meeting_json = if let Some(row) = meeting_row {
                    use sqlx::Row;
                    serde_json::json!({
                        "id": row.get::<Uuid, _>("id"),
                        "title": row.get::<String, _>("title"),
                        "date": row.get::<i64, _>("date"),
                    })
                } else {
                    serde_json::json!({
                        "id": mid.to_string(),
                        "title": "Unknown",
                        "date": 0,
                    })
                };

                results.push(KnowledgeSearchResult {
                    chunk: chunk_json,
                    meeting: meeting_json,
                    score: doc.score,
                });
            }
        }

        Ok(results)
    }
}

fn split_text(text: &str, size: usize, overlap: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let end = (i + size).min(words.len());
        let chunk = words[i..end].join(" ");
        if !chunk.trim().is_empty() {
            chunks.push(chunk.trim().to_string());
        }
        i += size - overlap;
    }
    chunks
}
