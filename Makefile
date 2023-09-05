include .env
.PHONY: up migrate

build:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml build

up:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d

migrate:
	docker-compose exec api sqlx migrate run --ignore-missing
	docker cp ./seeds/seed.sql quest-api_database_1:/tmp/
	docker exec quest-api_database_1 psql -U $(DATABASE_USER) -d $(DATABASE_DB) -q -f /tmp/seed.sql

start: up migrate

# ボリュームも合わせて削除する
down:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml down -v

test:
	cargo test
