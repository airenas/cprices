-include Makefile.options
POSTGRESQL_URL=postgres://editor:$(EDITOR_PASS)@localhost:5432/crypto?sslmode=disable
pair?=BTCUSDT,ETHUSDT
###############################################################################
run:
	RUST_LOG=DEBUG cargo run --bin importer -- --db-url $(POSTGRESQL_URL) --pair $(pair)
.PHONY: run
###############################################################################
run/build: build/local
	RUST_LOG=DEBUG target/release/importer --db-url $(POSTGRESQL_URL) --pair $(pair)
.PHONY: run/build
###############################################################################
build/local: 
	cargo build --release
.PHONY: build/local
###############################################################################
test/unit:
	RUST_LOG=DEBUG cargo test --no-fail-fast
.PHONY: test/unit		
test/lint:
	cargo clippy --all-targets --all-features -- -D warnings
.PHONY: test/lint	
###############################################################################
clean:
	rm -r -f target
.PHONY: clean
