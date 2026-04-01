-- companies (orgs)
CREATE TABLE IF NOT EXISTS companies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(64) UNIQUE NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_companies_slug ON companies(slug);

-- users
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    password_hash TEXT NOT NULL,
    created_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- company_members
CREATE TABLE IF NOT EXISTS company_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(16) NOT NULL DEFAULT 'member',
    joined_at BIGINT NOT NULL,
    UNIQUE(company_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_members_company ON company_members(company_id);
CREATE INDEX IF NOT EXISTS idx_members_user ON company_members(user_id);

-- company_invites
CREATE TABLE IF NOT EXISTS company_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    role VARCHAR(16) NOT NULL DEFAULT 'member',
    created_at BIGINT NOT NULL,
    used_at BIGINT,
    UNIQUE(company_id, email)
);
CREATE INDEX IF NOT EXISTS idx_invites_company ON company_invites(company_id);

-- member_api_keys
CREATE TABLE IF NOT EXISTS member_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(32) NOT NULL,
    key_encrypted TEXT NOT NULL,
    ollama_url VARCHAR(255),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE(company_id, user_id, provider)
);

-- company_config
CREATE TABLE IF NOT EXISTS company_config (
    company_id UUID PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
    allowed_models JSONB NOT NULL DEFAULT '[]',
    default_provider VARCHAR(32) NOT NULL DEFAULT 'anthropic',
    default_model VARCHAR(128) NOT NULL DEFAULT 'claude-sonnet-4-5',
    hivemind_enabled BOOLEAN NOT NULL DEFAULT true,
    updated_at BIGINT NOT NULL
);

-- meetings
CREATE TABLE IF NOT EXISTS meetings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    title VARCHAR(512) NOT NULL,
    date BIGINT NOT NULL,
    duration_seconds INT NOT NULL DEFAULT 0,
    participants JSONB NOT NULL DEFAULT '[]',
    summary TEXT,
    created_at BIGINT NOT NULL,
    vexa_meeting_id INT,
    vexa_platform VARCHAR(32),
    vexa_native_meeting_id VARCHAR(255)
);
CREATE INDEX IF NOT EXISTS idx_meetings_company_date ON meetings(company_id, date DESC);

-- knowledge_chunks
CREATE TABLE IF NOT EXISTS knowledge_chunks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    segment_id UUID,
    text TEXT NOT NULL,
    speaker VARCHAR(255),
    timestamp BIGINT,
    chunk_type VARCHAR(32) NOT NULL DEFAULT 'transcript',
    embedding JSONB,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_chunks_meeting ON knowledge_chunks(meeting_id);
CREATE INDEX IF NOT EXISTS idx_chunks_type ON knowledge_chunks(chunk_type);

-- FTS for keyword search
CREATE INDEX IF NOT EXISTS idx_chunks_text_gin ON knowledge_chunks USING GIN(to_tsvector('english', text));

-- token_usage
CREATE TABLE IF NOT EXISTS token_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id VARCHAR(64) NOT NULL,
    model VARCHAR(128) NOT NULL,
    provider VARCHAR(32) NOT NULL,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cost_cents INT NOT NULL DEFAULT 0,
    recorded_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_token_usage_company ON token_usage(company_id, recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_token_usage_user ON token_usage(user_id, company_id);

-- auth_tokens
CREATE TABLE IF NOT EXISTS auth_tokens (
    token VARCHAR(128) PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    created_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_auth_tokens_user ON auth_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_auth_tokens_company ON auth_tokens(company_id);
