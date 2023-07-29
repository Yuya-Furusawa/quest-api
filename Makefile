.PHONY: up migrate

build:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml build

up:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d

migrate:
	docker-compose exec api sqlx migrate run --ignore-missing

start: up migrate

# ボリュームも合わせて削除する
down:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml down -v

test:
	cargo test
