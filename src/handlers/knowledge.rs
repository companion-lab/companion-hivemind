use axum::extract::{Multipart, State};
use axum::response::Json;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthContext;
use crate::repos::knowledge::KnowledgeRepo;
use crate::services::knowledge::KnowledgeService;
use crate::services::pdf;
use crate::types::KnowledgeDocumentOut;

const MAX_PDF_SIZE: usize = 50 * 1024 * 1024; // 50MB

/// Upload a PDF document to the knowledge base.
/// Extracts text, chunks it, embeds vectors into Qdrant, and persists metadata.
pub async fn upload_pdf(
    State(state): State<AppState>,
    auth: AuthContext,
    mut multipart: Multipart,
) -> Result<Json<KnowledgeDocumentOut>, AppError> {
    let mut filename = String::from("unknown.pdf");
    let mut file_data = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            if let Some(fname) = field.file_name() {
                filename = fname.to_string();
            }
            file_data = field.bytes().await.map_err(|e| {
                AppError::BadRequest(format!("Failed to read file bytes: {}", e))
            })?.to_vec();

            if file_data.len() > MAX_PDF_SIZE {
                return Err(AppError::BadRequest(format!(
                    "File too large: {} bytes (max {} bytes)",
                    file_data.len(),
                    MAX_PDF_SIZE
                )));
            }
        }
    }

    if file_data.is_empty() {
        return Err(AppError::BadRequest("No file uploaded".into()));
    }

    if !filename.to_lowercase().ends_with(".pdf") {
        return Err(AppError::BadRequest("Only PDF files are supported".into()));
    }

    let now = crate::util::now_ms();
    let doc_id = uuid::Uuid::new_v4();

    // Create document record as "processing"
    let repo = KnowledgeRepo::new(state.db.clone());
    repo.create_document(
        doc_id,
        auth.company_id,
        auth.user_id,
        &filename,
        file_data.len() as i64,
        "processing",
        now,
    )
    .await?;

    // Extract text from PDF
    let text = pdf::extract_text_from_bytes(&file_data)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("PDF extraction failed: {}", e)))?;

    if text.trim().is_empty() {
        repo.update_document_status(doc_id, "empty", 0, now).await?;
        return Err(AppError::BadRequest("No text content found in PDF".into()));
    }

    // Chunk the text
    let docs = pdf::chunk_pdf_text(&text, auth.company_id, doc_id, &filename);
    let chunk_count = docs.len() as i32;

    // Ingest into Postgres + Qdrant
    KnowledgeService::ingest_pdf_documents(
        &state.db,
        &state.vector_store,
        auth.company_id,
        doc_id,
        &docs,
    )
    .await?;

    // Update document status to "completed"
    repo.update_document_status(doc_id, "completed", chunk_count, now).await?;

    // Return the document
    let doc = repo.get_document(doc_id, auth.company_id).await?
        .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    Ok(Json(doc))
}

/// List all knowledge documents for the company.
pub async fn list_documents(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<Vec<KnowledgeDocumentOut>>, AppError> {
    let repo = KnowledgeRepo::new(state.db.clone());
    let documents = repo.list_documents(auth.company_id).await?;
    Ok(Json(documents))
}

/// Delete a knowledge document and its vector embeddings.
pub async fn delete_document(
    State(state): State<AppState>,
    auth: AuthContext,
    axum::extract::Path(document_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<()>, AppError> {
    let repo = KnowledgeRepo::new(state.db.clone());

    // Verify document belongs to company
    let doc = repo.get_document(document_id, auth.company_id).await?
        .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    // Delete vector embeddings from Qdrant
    state.vector_store
        .delete_for_document(document_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to delete vectors: {}", e)))?;

    // Delete from Postgres
    repo.delete_document(document_id, auth.company_id).await?;

    tracing::info!(
        document_id = %document_id,
        company_id = %auth.company_id,
        filename = %doc.filename,
        "Knowledge document deleted"
    );

    Ok(Json(()))
}
