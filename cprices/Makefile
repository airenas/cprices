

###############################################################################
run:
	RUST_LOG=DEBUG cargo run --bin importer
.PHONY: run
###############################################################################
build/local: 
	cargo build --release
.PHONY: build/local
###############################################################################
test/unit:
	RUST_LOG=DEBUG cargo test
.PHONY: test/unit		
test/lint:
	cargo clippy --all-targets --all-features -- -D warnings
.PHONY: test/lint	
###############################################################################
clean:
	rm -r -f target
.PHONY: clean
