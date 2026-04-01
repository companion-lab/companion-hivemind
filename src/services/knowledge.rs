use uuid::Uuid;
use sqlx::{PgPool, Row};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::api::{TranscriptSegment, KnowledgeSearchResult};

const CHUNK_SIZE: usize = 400;
const CHUNK_OVERLAP: usize = 80;

pub struct KnowledgeService;

impl KnowledgeService {
    pub fn chunk_transcript(
        segments: &[TranscriptSegment],
        meeting_id: Uuid,
        meeting_date: i64,
    ) -> Vec<ChunkRow> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let mut chunks = Vec::new();

        for seg in segments {
            let sub_chunks = split_text(&seg.text, CHUNK_SIZE, CHUNK_OVERLAP);
            for text in sub_chunks {
                chunks.push(ChunkRow {
                    id: Uuid::new_v4(),
                    meeting_id,
                    segment_id: None,
                    text,
                    speaker: if seg.speaker.is_empty() { None } else { Some(seg.speaker.clone()) },
                    timestamp: Some(meeting_date + (seg.start_time * 1000.0) as i64),
                    chunk_type: "transcript".into(),
                    embedding: None,
                    metadata: serde_json::json!({
                        "start": seg.start_time,
                        "end": seg.end_time,
                    }),
                    created_at: now,
                });
            }
        }
        chunks
    }

    pub async fn search(
        db: &PgPool,
        company_id: Uuid,
        query: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<KnowledgeSearchResult>> {
        let rows = sqlx::query(
            r#"
            SELECT kc.id, kc.meeting_id, kc.text, kc.speaker, kc.timestamp,
                   kc.chunk_type, kc.metadata, m.title, m.date,
                   ts_rank(to_tsvector('english', kc.text), plainto_tsquery('english', $3)) as score
            FROM knowledge_chunks kc
            JOIN meetings m ON m.id = kc.meeting_id
            WHERE m.company_id = $1
              AND to_tsvector('english', kc.text) @@ plainto_tsquery('english', $3)
            ORDER BY score DESC
            LIMIT $2
            "#,
        )
        .bind(company_id)
        .bind(limit as i64)
        .bind(query)
        .fetch_all(db)
        .await?;

        let results = rows
            .into_iter()
            .map(|r| {
                let score: Option<f64> = r.try_get("score").ok().or(Some(0.0));
                KnowledgeSearchResult {
                    chunk: serde_json::json!({
                        "id": r.get::<Uuid, _>("id"),
                        "meeting_id": r.get::<Uuid, _>("meeting_id"),
                        "text": r.get::<String, _>("text"),
                        "speaker": r.try_get::<Option<String>, _>("speaker").ok().flatten(),
                        "timestamp": r.try_get::<Option<i64>, _>("timestamp").ok().flatten(),
                        "chunk_type": r.get::<String, _>("chunk_type"),
                        "metadata": r.get::<serde_json::Value, _>("metadata"),
                    }),
                    meeting: serde_json::json!({
                        "id": r.get::<Uuid, _>("meeting_id"),
                        "title": r.get::<String, _>("title"),
                        "date": r.get::<i64, _>("date"),
                    }),
                    score: score.unwrap_or(0.0),
                }
            })
            .collect();

        Ok(results)
    }
}

#[derive(Debug)]
pub struct ChunkRow {
    pub id: Uuid,
    pub meeting_id: Uuid,
    #[allow(dead_code)]
    pub segment_id: Option<Uuid>,
    pub text: String,
    pub speaker: Option<String>,
    pub timestamp: Option<i64>,
    pub chunk_type: String,
    #[allow(dead_code)]
    pub embedding: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    pub created_at: i64,
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
