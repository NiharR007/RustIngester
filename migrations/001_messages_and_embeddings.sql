-- Enable pgvector extension for efficient vector operations
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Conversations table to store conversation metadata
CREATE TABLE IF NOT EXISTS conversations (
    conversation_id UUID PRIMARY KEY,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Messages table to store full message content
CREATE TABLE IF NOT EXISTS messages (
    message_id UUID PRIMARY KEY,
    conversation_id UUID NOT NULL REFERENCES conversations(conversation_id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Message embeddings with pgvector (768-dim Nomic embeddings)
CREATE TABLE IF NOT EXISTS message_embeddings (
    message_id UUID PRIMARY KEY REFERENCES messages(message_id) ON DELETE CASCADE,
    embedding vector(768) NOT NULL,
    embedding_model VARCHAR(100) DEFAULT 'nomic-embed-text-v1.5',
    created_at TIMESTAMP DEFAULT NOW()
);

-- Knowledge graph nodes (from enhanced pipeline)
CREATE TABLE IF NOT EXISTS kg_nodes (
    node_id VARCHAR(255),
    conversation_id UUID REFERENCES conversations(conversation_id) ON DELETE CASCADE,
    node_type VARCHAR(100),
    created_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (node_id, conversation_id)
);

-- Knowledge graph edges with evidence message IDs
CREATE TABLE IF NOT EXISTS kg_edges (
    edge_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    conversation_id UUID REFERENCES conversations(conversation_id) ON DELETE CASCADE,
    source_node VARCHAR(255) NOT NULL,
    target_node VARCHAR(255) NOT NULL,
    relation VARCHAR(255) NOT NULL,
    evidence_message_ids UUID[] NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_message_embeddings_ivfflat ON message_embeddings 
    USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX IF NOT EXISTS idx_kg_edges_conversation ON kg_edges(conversation_id);
CREATE INDEX IF NOT EXISTS idx_kg_edges_evidence ON kg_edges USING GIN(evidence_message_ids);
CREATE INDEX IF NOT EXISTS idx_kg_nodes_conversation ON kg_nodes(conversation_id);
CREATE INDEX IF NOT EXISTS idx_kg_nodes_type ON kg_nodes(node_type);

