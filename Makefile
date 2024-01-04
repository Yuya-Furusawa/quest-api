include .env
.PHONY: up migrate seed

build:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml build

up:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d

migrate:
	docker-compose exec api sqlx migrate run --ignore-missing

seed:
	docker cp ./seeds/seed.sql quest-database-container:/tmp/
	docker exec quest-database-container psql -U $(DATABASE_USER) -d $(DATABASE_DB) -q -f /tmp/seed.sql

start: up migrate seed

# ボリュームも合わせて削除する
down:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml down -v

test:
	cargo test
