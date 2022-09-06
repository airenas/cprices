-include .env
sql_dir=../../sql
POSTGRESQL_URL=postgres://editor:$(EDITOR_PASS)@localhost:5432/crypto?sslmode=disable

start: postgresql/init.sql
	docker compose up -d
.PHONY: start

postgresql/init.sql: postgresql/init.sql.in
	cat $^ | envsubst > $@

psql/root:
	docker compose exec -e PGPASSWORD=$(ROOT_PASS) timescaledb psql -U postgres -h localhost -d crypto
.PHONY: psql/root
psql/editor:
	docker compose exec -e PGPASSWORD=$(EDITOR_PASS) timescaledb psql -U editor -h localhost -d crypto
.PHONY: psql/editor

migrate/install: 
	go install -tags 'postgres' github.com/golang-migrate/migrate/v4/cmd/migrate@latest

migrate/new: 
	migrate create -ext sql -dir $(sql_dir) -seq $(name)

migrate/up: 
	migrate -database $(POSTGRESQL_URL) -path $(sql_dir) up	


.EXPORT_ALL_VARIABLES:	