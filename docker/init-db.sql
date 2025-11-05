-- Initialize PostgreSQL with required extensions for RustIngester

-- Enable vector extension (for pgvector)
CREATE EXTENSION IF NOT EXISTS vector;

-- Enable uuid extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Note: Apache AGE requires manual installation in the base image
-- For now, we'll skip AGE in Docker and rely on pgvector for core functionality

-- Create schema for application tables
CREATE SCHEMA IF NOT EXISTS ag_catalog;

-- Set search path
SET search_path = ag_catalog, "$user", public;

-- Grant permissions
GRANT ALL PRIVILEGES ON SCHEMA ag_catalog TO postgres;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA ag_catalog TO postgres;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA ag_catalog TO postgres;

-- Success message
DO $$
BEGIN
    RAISE NOTICE 'RustIngester database initialized successfully!';
    RAISE NOTICE 'Extensions: vector, uuid-ossp';
END $$;
