use std::collections::HashMap;

use crate::services::knowledge::split_text;

const PDF_CHUNK_SIZE: usize = 400;
const PDF_CHUNK_OVERLAP: usize = 80;

/// Extract text content from a PDF file's raw bytes.
pub fn extract_text_from_bytes(data: &[u8]) -> anyhow::Result<String> {
    let text = pdf_extract::extract_text_from_mem(data)
        .map_err(|e| anyhow::anyhow!("Failed to extract text from PDF: {}", e))?;
    Ok(text)
}

/// Chunk extracted PDF text into knowledge documents with metadata.
pub fn chunk_pdf_text(
    text: &str,
    company_id: uuid::Uuid,
    document_id: uuid::Uuid,
    document_name: &str,
) -> Vec<langchain_rust::schemas::Document> {
    let chunks = split_text(text, PDF_CHUNK_SIZE, PDF_CHUNK_OVERLAP);
    let mut docs = Vec::new();

    for (i, chunk_text) in chunks.into_iter().enumerate() {
        let mut metadata: HashMap<String, serde_json::Value> = HashMap::new();
        metadata.insert(
            "company_id".to_string(),
            serde_json::json!(company_id.to_string()),
        );
        metadata.insert(
            "document_id".to_string(),
            serde_json::json!(document_id.to_string()),
        );
        metadata.insert(
            "document_name".to_string(),
            serde_json::json!(document_name),
        );
        metadata.insert(
            "chunk_type".to_string(),
            serde_json::json!("pdf_document"),
        );
        metadata.insert("chunk_index".to_string(), serde_json::json!(i));

        docs.push(langchain_rust::schemas::Document {
            page_content: chunk_text,
            metadata,
            score: 0.0,
        });
    }

    docs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_pdf_text() {
        let company_id = uuid::Uuid::new_v4();
        let document_id = uuid::Uuid::new_v4();
        let text = "This is a test document. ".repeat(100);
        let docs = chunk_pdf_text(&text, company_id, document_id, "test.pdf");
        assert!(!docs.is_empty());
        for doc in &docs {
            assert_eq!(
                doc.metadata.get("chunk_type").unwrap().as_str().unwrap(),
                "pdf_document"
            );
            assert_eq!(
                doc.metadata
                    .get("document_id")
                    .unwrap()
                    .as_str()
                    .unwrap(),
                document_id.to_string()
            );
        }
    }
}
