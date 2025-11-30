up:
	docker compose -f ops/docker-compose.dev.yml up -d
	$(MAKE) migrate

down:
	docker compose -f ops/docker-compose.dev.yml down

logs:
	docker compose -f ops/docker-compose.dev.yml logs -f

bootstrap:
	rustup toolchain install 1.91.1 || true
	rustup default 1.91.1 || true
	cd backend && cargo fetch || true

test:
	cd backend && cargo test --all

test-unit:
	cd backend && cargo test --test '*' -- --nocapture

test-integration:
	cd backend && cargo test --test '*' -- --nocapture

start-deps:
	docker compose -f ops/docker-compose.dev.yml up -d

start-backend:
	cd backend && cargo run --bin mnemo-api-service

start-frontend:
	cd ui && npm run dev

run:
	make start-deps
	( make start-backend & )
	( make start-frontend & )
	wait

migrate:
	cd backend && (command -v sqlx >/dev/null 2>&1 || cargo install sqlx-cli --no-default-features --features postgres) && \
	psql "$${DATABASE_URL:-postgres://mnemo:mnemo@localhost:5432/mnemo}" -c "DROP TABLE IF EXISTS _sqlx_migrations;" >/dev/null 2>&1 || true ; \
	sqlx migrate run --source migrations/postgres
