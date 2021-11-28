PROJDIR=$(shell git rev-parse --show-toplevel)

.PHONY: cli release

release:
	@cd cli && cargo build --release
	@cd desktop && npm run build && cargo tauri build

cli:
	@cargo install --path cli

dbgcore:
	@cargo test test_core_debug -- --nocapture
