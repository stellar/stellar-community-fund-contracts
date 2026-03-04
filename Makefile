lint-check-contracts:
	cd contracts && cargo clippy -- -D warnings

lint-check-neurons:
	cd neurons && cargo clippy --target wasm32-unknown-unknown -- -D warnings

fmt-check-contracts:
	cd contracts && cargo fmt --check

fmt-check-neurons:
	cd neurons && cargo fmt --check

fmt-fix:
	cd contracts && cargo fmt
	cd neurons && cargo fmt