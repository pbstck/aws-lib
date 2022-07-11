lint/check:
	@echo "check linting rules..."
	cargo fmt --all -- --check
	cargo clippy -- -D warnings

lint/format:
	@echo "applying linting rules..."
	cargo fmt --all
	cargo clippy -- -D warnings