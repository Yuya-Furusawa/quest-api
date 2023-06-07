include .env

build:
	docker-compose build

db:
	docker-compose up

dev:
	docker-compose exec api bash -c "sqlx db create --database-url $(DATABASE_URL)"
	docker-compose exec api bash -c "sqlx migrate run --ignore-missing"
	docker-compose exec api cargo watch -x run

test:
	cargo test
