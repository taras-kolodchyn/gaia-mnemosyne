up:
	docker compose -f ops/docker-compose.dev.yml up -d

down:
	docker compose -f ops/docker-compose.dev.yml down

logs:
	docker compose -f ops/docker-compose.dev.yml logs -f

bootstrap:
	npm install --prefix ui || true
	cargo fetch || true