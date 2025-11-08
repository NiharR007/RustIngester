-- Initialize PostgreSQL with required extensions for RustIngester

-- Enable vector extension (for pgvector)
CREATE EXTENSION IF NOT EXISTS vector;

-- Enable uuid extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable Apache AGE extension
CREATE EXTENSION IF NOT EXISTS age;

-- Load AGE into the current session
LOAD 'age';

-- Create schema for application tables
CREATE SCHEMA IF NOT EXISTS ag_catalog;

-- Set search path to include ag_catalog
SET search_path = ag_catalog, "$user", public;

-- Grant permissions
GRANT ALL PRIVILEGES ON SCHEMA ag_catalog TO postgres;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA ag_catalog TO postgres;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA ag_catalog TO postgres;

-- Create the knowledge graph (if it doesn't exist)
DO $$
BEGIN
    -- Check if graph exists
    IF NOT EXISTS (
        SELECT 1 FROM ag_catalog.ag_graph WHERE name = 'sem_graph'
    ) THEN
        PERFORM ag_catalog.create_graph('sem_graph');
        RAISE NOTICE 'Created knowledge graph: sem_graph';
    ELSE
        RAISE NOTICE 'Knowledge graph sem_graph already exists';
    END IF;
END $$;

-- Success message
DO $$
BEGIN
    RAISE NOTICE '✅ RustIngester database initialized successfully!';
    RAISE NOTICE 'Extensions: vector, uuid-ossp, age';
    RAISE NOTICE 'Knowledge Graph: sem_graph';
END $$;
