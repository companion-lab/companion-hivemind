-- knowledge_documents: tracks uploaded PDF documents
CREATE TABLE IF NOT EXISTS knowledge_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename VARCHAR(512) NOT NULL,
    content_type VARCHAR(128) NOT NULL DEFAULT 'application/pdf',
    file_size BIGINT NOT NULL DEFAULT 0,
    text_content TEXT,
    status VARCHAR(32) NOT NULL DEFAULT 'processing',
    chunk_count INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_knowledge_docs_company ON knowledge_documents(company_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_docs_user ON knowledge_documents(user_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_docs_status ON knowledge_documents(status);

-- Update knowledge_chunks to allow NULL meeting_id (for PDF documents)
ALTER TABLE knowledge_chunks
    ALTER COLUMN meeting_id DROP NOT NULL;

ALTER TABLE knowledge_chunks
    ADD COLUMN IF NOT EXISTS document_id UUID REFERENCES knowledge_documents(id) ON DELETE CASCADE;

ALTER TABLE knowledge_chunks
    ADD COLUMN IF NOT EXISTS segment_id UUID;

CREATE INDEX IF NOT EXISTS idx_chunks_document ON knowledge_chunks(document_id);
