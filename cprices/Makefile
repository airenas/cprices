

run:
	RUST_LOG=DEBUG cargo run --bin importer
.PHONY: run	

test/unit:
	RUST_LOG=DEBUG cargo test
.PHONY: test/unit		

test/lint:
	cargo clippy --all-targets --all-features -- -D warnings
.PHONY: test/lint	
