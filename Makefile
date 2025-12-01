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
	URL="$${DATABASE_URL:-postgres://mnemo:mnemo@127.0.0.1:5432/mnemo}" ; \
	printf "Waiting for Postgres at %s ...\n" "$$URL" ; \
	for i in $$(seq 1 30); do \
		if psql "$$URL" -c "SELECT 1" >/dev/null 2>&1; then \
			printf "Postgres is ready\n" ; \
			break ; \
		fi ; \
		sleep 2 ; \
		if [ $$i -eq 30 ]; then \
			echo "Postgres not ready after waiting" ; \
			exit 1 ; \
		fi ; \
	done ; \
	psql "$$URL" -c "DROP TABLE IF EXISTS _sqlx_migrations;" >/dev/null 2>&1 || true ; \
	sqlx migrate run --source migrations/postgres --database-url "$$URL"
