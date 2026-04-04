use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::types::KnowledgeDocumentOut;

pub struct KnowledgeRepo {
    db: PgPool,
}

impl KnowledgeRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Create a new knowledge document record.
    pub async fn create_document(
        &self,
        id: Uuid,
        company_id: Uuid,
        user_id: Uuid,
        filename: &str,
        file_size: i64,
        status: &str,
        now: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO knowledge_documents (id, company_id, user_id, filename, content_type, file_size, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(id)
        .bind(company_id)
        .bind(user_id)
        .bind(filename)
        .bind("application/pdf")
        .bind(file_size)
        .bind(status)
        .bind(now)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }

    /// Update document status and chunk count after processing.
    pub async fn update_document_status(
        &self,
        id: Uuid,
        status: &str,
        chunk_count: i32,
        now: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE knowledge_documents SET status = $1, chunk_count = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(status)
        .bind(chunk_count)
        .bind(now)
        .bind(id)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }

    /// Get a single document by ID, scoped to company.
    pub async fn get_document(
        &self,
        id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<KnowledgeDocumentOut>, AppError> {
        let row = sqlx::query(
            "SELECT id, company_id, user_id, filename, content_type, file_size, status, chunk_count, metadata, created_at, updated_at FROM knowledge_documents WHERE id = $1 AND company_id = $2",
        )
        .bind(id)
        .bind(company_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::from)?;

        match row {
            Some(row) => Ok(Some(self.map_document_row(row))),
            None => Ok(None),
        }
    }

    /// List all knowledge documents for a company.
    pub async fn list_documents(
        &self,
        company_id: Uuid,
    ) -> Result<Vec<KnowledgeDocumentOut>, AppError> {
        let rows = sqlx::query(
            "SELECT id, company_id, user_id, filename, content_type, file_size, status, chunk_count, metadata, created_at, updated_at FROM knowledge_documents WHERE company_id = $1 ORDER BY created_at DESC",
        )
        .bind(company_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(|r| self.map_document_row(r)).collect())
    }

    /// Delete a knowledge document (cascades to chunks).
    pub async fn delete_document(&self, id: Uuid, company_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            "DELETE FROM knowledge_documents WHERE id = $1 AND company_id = $2",
        )
        .bind(id)
        .bind(company_id)
        .execute(&self.db)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }

    fn map_document_row(&self, row: sqlx::postgres::PgRow) -> KnowledgeDocumentOut {
        use sqlx::Row;
        KnowledgeDocumentOut {
            id: row.get("id"),
            company_id: row.get("company_id"),
            user_id: row.get("user_id"),
            filename: row.get("filename"),
            content_type: row.get("content_type"),
            file_size: row.get("file_size"),
            status: row.get("status"),
            chunk_count: row.get("chunk_count"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
