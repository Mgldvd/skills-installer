SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(PATH)

FRONTEND_DIR := frontend
TAURI_DIR := src-tauri
TAURI_BIN := $(FRONTEND_DIR)/node_modules/.bin/tauri
DIST_DIR := dist
VERSION := $(shell grep -m1 '^version' $(TAURI_DIR)/Cargo.toml | cut -d '"' -f2)

.PHONY: dev typecheck lint test build appimage deb release clean frontend-deps

# One-time / as-needed frontend dependency install. Cheap to re-run: npm
# no-ops when node_modules is already up to date.
frontend-deps:
	cd $(FRONTEND_DIR) && npm install

## Start the Tauri development app (hot-reloading Vue frontend + Rust backend).
dev: frontend-deps
	$(TAURI_BIN) dev

## Type-check both halves of the app without emitting anything.
typecheck: frontend-deps
	cd $(FRONTEND_DIR) && npm run typecheck
	cargo check --manifest-path $(TAURI_DIR)/Cargo.toml --all-targets

## Lint both halves of the app.
lint: frontend-deps
	cd $(FRONTEND_DIR) && npm run lint
	cargo fmt --manifest-path $(TAURI_DIR)/Cargo.toml -- --check
	cargo clippy --manifest-path $(TAURI_DIR)/Cargo.toml --all-targets -- -D warnings

## Run both test suites. No real network calls or installs happen here —
## Rust tests use FakeInstaller/FakeProcessRunner, Vue tests use jsdom.
test: frontend-deps
	cd $(FRONTEND_DIR) && npm run test
	cargo test --manifest-path $(TAURI_DIR)/Cargo.toml

## Build the release-grade raw executable and the configured bundles
## (AppImage + .deb, per src-tauri/tauri.conf.json's bundle.targets).
build: frontend-deps
	$(TAURI_BIN) build

## Copy the built AppImage into dist/ with the documented naming pattern.
appimage: build
	mkdir -p $(DIST_DIR)
	@src=$$(find $(TAURI_DIR)/target/release/bundle/appimage -maxdepth 1 -name '*.AppImage' | head -n1); \
	if [ -n "$$src" ]; then \
		cp "$$src" "$(DIST_DIR)/Skills-Installer-$(VERSION)-x86_64.AppImage"; \
		echo "Copied $$src -> $(DIST_DIR)/Skills-Installer-$(VERSION)-x86_64.AppImage"; \
	else \
		echo "No AppImage found under $(TAURI_DIR)/target/release/bundle/appimage" >&2; exit 1; \
	fi

## Copy the built .deb into dist/ with the documented naming pattern.
deb: build
	mkdir -p $(DIST_DIR)
	@src=$$(find $(TAURI_DIR)/target/release/bundle/deb -maxdepth 1 -name '*.deb' | head -n1); \
	if [ -n "$$src" ]; then \
		cp "$$src" "$(DIST_DIR)/skills-installer_$(VERSION)_amd64.deb"; \
		echo "Copied $$src -> $(DIST_DIR)/skills-installer_$(VERSION)_amd64.deb"; \
	else \
		echo "No .deb found under $(TAURI_DIR)/target/release/bundle/deb" >&2; exit 1; \
	fi

## Full verification pipeline plus every Linux artifact, copied into dist/.
release: typecheck lint test appimage deb
	mkdir -p $(DIST_DIR)
	cp $(TAURI_DIR)/target/release/skills-installer $(DIST_DIR)/skills-installer
	@echo "Release artifacts:"
	@ls -la $(DIST_DIR)/

clean:
	rm -rf $(TAURI_DIR)/target $(FRONTEND_DIR)/dist $(DIST_DIR)
