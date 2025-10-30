# Database Setup Guide

## Current Issue

The ingestion is failing with: `FATAL: password authentication failed for user "postgres"`

## Solution

You need to configure the correct database credentials in your `.env` file.

### Option 1: Use Trust Authentication (Development Only)

If you're running PostgreSQL locally for development, you can configure it to trust local connections:

1. Find your `pg_hba.conf` file:
```bash
psql -U postgres -c "SHOW hba_file;"
```

2. Edit the file and change the authentication method to `trust` for local connections:
```
# TYPE  DATABASE        USER            ADDRESS                 METHOD
local   all             all                                     trust
host    all             all             127.0.0.1/32            trust
host    all             all             ::1/128                 trust
```

3. Reload PostgreSQL:
```bash
# macOS with Homebrew
brew services restart postgresql@14

# Linux
sudo systemctl restart postgresql
```

4. Update your `.env` file:
```bash
DATABASE_URL=postgresql://postgres@localhost:5432/postgres
LSH_BUCKETS=128
SERVER_PORT=3000
```

### Option 2: Set a Password (Recommended)

1. Set a password for the postgres user:
```bash
psql -U postgres -c "ALTER USER postgres PASSWORD 'your_password';"
```

2. Update your `.env` file with the password:
```bash
DATABASE_URL=postgresql://postgres:your_password@localhost:5432/postgres
LSH_BUCKETS=128
SERVER_PORT=3000
```

### Option 3: Create a New User

1. Create a new database user:
```bash
psql -U postgres << EOF
CREATE USER rust_ingester WITH PASSWORD 'secure_password';
CREATE DATABASE rust_ingester_db OWNER rust_ingester;
GRANT ALL PRIVILEGES ON DATABASE rust_ingester_db TO rust_ingester;
EOF
```

2. Enable AGE extension on the new database:
```bash
psql -U postgres -d rust_ingester_db << EOF
CREATE EXTENSION IF NOT EXISTS age;
LOAD 'age';
SET search_path = ag_catalog, "\$user", public;
EOF
```

3. Update your `.env` file:
```bash
DATABASE_URL=postgresql://rust_ingester:secure_password@localhost:5432/rust_ingester_db
LSH_BUCKETS=128
SERVER_PORT=3000
```

## Quick Test

After updating your `.env` file, test the connection:

```bash
# Test with psql
psql "$DATABASE_URL" -c "SELECT version();"

# Test the ingestion again
cargo run --release --bin ingest_cli -- Data/ok.json
```

## Current .env File Location

Your `.env` file should be at:
```
/Users/niharpatel/Desktop/RustIngester/.env
```

## Example .env File

Create or update this file with one of the configurations above:

```bash
# Option 1: No password (trust authentication)
DATABASE_URL=postgresql://postgres@localhost:5432/postgres

# Option 2: With password
DATABASE_URL=postgresql://postgres:mypassword@localhost:5432/postgres

# Option 3: Custom user and database
DATABASE_URL=postgresql://rust_ingester:secure_password@localhost:5432/rust_ingester_db

# Other settings
LSH_BUCKETS=128
SERVER_PORT=3000
```

## Verify Setup

Once configured, verify everything works:

```bash
# 1. Test database connection
psql "$DATABASE_URL" -c "SELECT version();"

# 2. Check AGE extension
psql "$DATABASE_URL" -c "SELECT * FROM pg_extension WHERE extname='age';"

# 3. Run the ingestion
cargo run --release --bin ingest_cli -- Data/ok.json
```

## Troubleshooting

### "FATAL: database does not exist"
Create the database:
```bash
createdb -U postgres rust_ingester_db
```

### "extension 'age' does not exist"
Install AGE extension:
```bash
cd age
make PG_CONFIG=$(which pg_config) install
psql -U postgres -d your_database -c "CREATE EXTENSION age;"
```

### "connection refused"
Start PostgreSQL:
```bash
# macOS
brew services start postgresql@14

# Linux
sudo systemctl start postgresql
```

### Check PostgreSQL is running
```bash
pg_isready
ps aux | grep postgres
```
