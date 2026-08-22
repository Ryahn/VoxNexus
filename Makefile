COMPOSE ?= docker compose
RUN := $(COMPOSE) run --rm app
# SHELL := powershell.exe
# .SHELLFLAGS := -NoProfile -Command

help:
	@echo "  make check       Check the codebase"

check:
	@echo "cargo fmt check"
	cargo fmt --all -- --check
	@echo "cargo clippy check"
	cargo clippy --workspace --all-targets -- -D warnings
	@echo "cargo test workspace"
	cargo test --workspace
	@echo "pnpm build"
	pnpm build
	@echo "pnpm check-codegen"
	pnpm check-codegen
	@echo "pnpm lint"
	pnpm lint
