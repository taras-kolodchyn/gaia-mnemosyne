-- Core Gaia Mnemosyne schema (PostgreSQL)

-- Fingerprints for incremental ingestion
CREATE TABLE IF NOT EXISTS fingerprints (
    path TEXT PRIMARY KEY,
    hash TEXT NOT NULL
);

-- Files (documents)
CREATE TABLE IF NOT EXISTS files (
    id SERIAL PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    namespace TEXT,
    modified_at TIMESTAMPTZ,
    file_size BIGINT,
    file_type TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Chunks
CREATE TABLE IF NOT EXISTS chunks (
    id SERIAL PRIMARY KEY,
    file_id INTEGER REFERENCES files(id) ON DELETE CASCADE,
    idx INTEGER NOT NULL,
    tags TEXT[],
    vector_id TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(file_id, idx)
);

-- Mapping file -> chunk indices
CREATE TABLE IF NOT EXISTS file_chunks(
    file_id INTEGER REFERENCES files(id) ON DELETE CASCADE,
    chunk_id INTEGER REFERENCES chunks(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    PRIMARY KEY(file_id, chunk_id)
);

-- Jobs
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY,
    job_type TEXT,
    status TEXT,
    progress INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- File dependency graph (language-agnostic)
CREATE TABLE IF NOT EXISTS file_dependencies(
    file_path TEXT NOT NULL,
    dep_path TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY(file_path, dep_path)
);

-- Ontology rules
CREATE TABLE IF NOT EXISTS ontology_rules (
    id SERIAL PRIMARY KEY,
    tag TEXT NOT NULL,
    patterns TEXT[] NOT NULL
);

-- RAG sessions
CREATE TABLE IF NOT EXISTS rag_sessions(
    id UUID PRIMARY KEY,
    history JSONB,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Semantic clusters
CREATE TABLE IF NOT EXISTS semantic_clusters(
    doc_id TEXT NOT NULL,
    cluster_id INT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY(doc_id, cluster_id)
);

-- Seed ontology defaults (idempotent)
INSERT INTO ontology_rules(tag, patterns)
VALUES
    ('project', ARRAY['project', 'repo', 'repository']),
    ('domain', ARRAY['domain', 'api', 'service']),
    ('company', ARRAY['company', 'org', 'enterprise'])
ON CONFLICT DO NOTHING;
