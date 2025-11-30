# Gaia Mnemosyne Ops

Docker-compose and scripts for local development stack.

## Services (dev)
- **mnemo-api-service** (built from backend)
- **Qdrant** (vector DB) on 6333
- **SurrealDB 2.x** on 8000
- **Redis** on 6379
- **Postgres** on 5432

## Usage
```bash
# start stack
make up
# stop
make down
# logs
make logs
```

### docker-compose
File: `ops/docker-compose.dev.yml`
- Mounts backend source into container for builds
- Optional host mount: `/mnt/host_downloads` for filesystem ingestion roots

## Environment
- SURREALDB_USER / SURREALDB_PASS (default root/root)
- QDRANT_URL (default http://qdrant:6333)
- DATABASE_URL (postgres://mnemo:mnemo@postgres:5432/mnemo)
- INGESTION_ROOT (e.g., /mnt/host_downloads)

## Troubleshooting
- Surreal 400: ensure using `surrealdb/surrealdb:latest` (2.x) and hashed IDs
- Postgres volume issues: clear `postgres_data` volume if major version bumped
- Qdrant collection: created automatically as `mnemo_chunks`
