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

.DEFAULT_GOAL := build

.PHONY: build test lint lint-all-features lint-nightly lint-nightly-all-features \
	fmt fmt-check tool-versions setup-devenv

build:
	cargo +$(STABLE) build --locked $(ARGS)

test:
	cargo +$(STABLE) test --locked $(ARGS)

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

tool-versions:
	@printf '%s\n' \
		'stable=$(STABLE)' \
		'nightly=$(NIGHTLY)'

setup-devenv:
	rustup toolchain install $(STABLE) --profile minimal --component clippy --component rustfmt
	rustup toolchain install $(NIGHTLY) --profile minimal --component clippy --component rustfmt
