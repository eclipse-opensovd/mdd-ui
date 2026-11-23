# SPDX-FileCopyrightText: 2026 Copyright (c) Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0

STABLE := 1.88.0
NIGHTLY := nightly-2026-07-21
CARGO_DENY_VERSION := 0.20.2
TAURI_CLI_VERSION := 2.10.1
UV_VERSION := 0.13.6

# tools installed by setup-devenv (uv) are placed in ~/.local/bin
export PATH := $(HOME)/.local/bin:$(PATH)

.DEFAULT_GOAL := build

.PHONY: build test lint lint-all-features lint-nightly lint-nightly-all-features \
	tauri-dev tauri-build tauri-build-unsigned fmt fmt-check precommit precommit-all-features deny tool-versions \
	setup-devenv install-tools install-uv doctor help

build:
	cargo +$(STABLE) build --locked $(ARGS)

test:
	cargo +$(STABLE) test --locked $(ARGS)

# development, with hot-reload
tauri-dev:
	cargo +$(STABLE) tauri dev $(ARGS)

# release build
tauri-build:
	cargo +$(STABLE) tauri build $(ARGS)

# release build without code signing (no updater artifacts)
tauri-build-unsigned:
	cargo +$(STABLE) tauri build --config '{"bundle":{"createUpdaterArtifacts":false}}' $(ARGS)

lint:
	cargo +$(STABLE) clippy --all-targets $(ARGS) -- --deny=warnings

lint-all-features:
	$(MAKE) lint ARGS="--all-features $(ARGS)"

lint-nightly:
	cargo +$(NIGHTLY) clippy --all-targets $(ARGS) -- --deny=warnings

lint-nightly-all-features:
	$(MAKE) lint-nightly ARGS="--all-features $(ARGS)"

fmt:
	cargo +$(NIGHTLY) fmt $(ARGS)

fmt-check:
	$(MAKE) fmt ARGS="--check $(ARGS)"

precommit:
	uv run --locked --group tools prek run --all-files --show-diff-on-failure

precommit-all-features:
	uv run --locked --group tools prek run --all-files --group CI --group @ungrouped --show-diff-on-failure

deny:
	cargo +$(STABLE) deny check $(ARGS)

tool-versions:
	@printf '%s\n' \
		'stable=$(STABLE)' \
		'nightly=$(NIGHTLY)' \
		'cargo_deny=$(CARGO_DENY_VERSION)'

setup-devenv:
	@command -v rustup >/dev/null 2>&1 || { printf '%s\n' 'rustup is required: https://rustup.rs'; exit 1; }
	$(MAKE) install-uv
	rustup toolchain install $(STABLE) --profile minimal --component clippy --component rustfmt
	rustup toolchain install $(NIGHTLY) --profile minimal --component clippy --component rustfmt
	$(MAKE) install-tools
	cargo +$(STABLE) install --locked --version $(TAURI_CLI_VERSION) tauri-cli
	uv sync --locked --group tools
	uv run --locked --group tools prek install
	$(MAKE) doctor

install-tools:
	cargo +$(STABLE) install --locked --version $(CARGO_DENY_VERSION) cargo-deny

install-uv:
	@command -v uv >/dev/null 2>&1 || curl -LsSf https://astral.sh/uv/$(UV_VERSION)/install.sh | sh

doctor:
	@missing=0; \
	for tool in rustup cargo uv git cargo-deny; do \
		if command -v "$$tool" >/dev/null 2>&1; then \
			printf 'Found required tool: %s\n' "$$tool"; \
		else \
			printf 'Missing required tool: %s\n' "$$tool"; \
			missing=1; \
		fi; \
	done; \
	if command -v rustup >/dev/null 2>&1; then \
		rustup run $(STABLE) rustc --version >/dev/null 2>&1 || { printf 'Missing Rust toolchain: %s\n' '$(STABLE)'; missing=1; }; \
		rustup run $(NIGHTLY) rustc --version >/dev/null 2>&1 || { printf 'Missing Rust toolchain: %s\n' '$(NIGHTLY)'; missing=1; }; \
	fi; \
	if command -v cargo-deny >/dev/null 2>&1; then \
		version=$$(cargo-deny --version 2>/dev/null); \
		if [ "$$version" != 'cargo-deny $(CARGO_DENY_VERSION)' ]; then \
			printf 'Expected cargo-deny %s, found %s\n' '$(CARGO_DENY_VERSION)' "$$version"; \
			missing=1; \
		fi; \
	fi; \
	if [ "$$missing" -ne 0 ]; then exit 1; fi
	@uv run --locked --group tools prek --version

help:
	@printf '%s\n' \
		'build ARGS="..."             Build the workspace with optional cargo arguments' \
		'test ARGS="..."              Run tests' \
		'tauri-dev ARGS="..."         Run the app in development mode with hot-reload' \
		'tauri-build ARGS="..."       Release build' \
		'tauri-build-unsigned         Release build without code signing' \
		'lint ARGS="..."              Run stable Clippy' \
		'lint-all-features            Run stable Clippy with all features' \
		'lint-nightly ARGS="..."      Run pinned-nightly Clippy' \
		'lint-nightly-all-features    Run pinned-nightly Clippy with all features' \
		'fmt ARGS="..."               Format code' \
		'fmt-check                    Check code formatting' \
		'precommit                    Run standard pre-commit checks' \
		'precommit-all-features       Run all pre-commit checks (as in CI)' \
		'deny ARGS="..."              Run all or selected cargo-deny checks' \
		'tool-versions                Print machine-readable tool versions' \
		'setup-devenv                 Install pinned development tools and pre-commit hooks' \
		'install-tools                Install pinned cargo tools (cargo-deny)' \
		'install-uv                   Install uv if missing' \
		'doctor                       Check development prerequisites'
