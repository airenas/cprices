-include Makefile.options
POSTGRESQL_URL=postgres://editor:$(EDITOR_PASS)@localhost:5432/crypto?sslmode=disable
###############################################################################
run:
	RUST_LOG=DEBUG cargo run --bin importer -- --db-url $(POSTGRESQL_URL)
.PHONY: run
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
