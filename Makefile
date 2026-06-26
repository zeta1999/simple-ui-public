# Extended Markdown GUI - Makefile

.PHONY: all build test clean run-tui dev-graphical build-graphical

# Default target
all: build test

# Build everything
build: build-rust build-wasm build-graphical

# Rust Commands
build-rust:
	cargo build --all

test:
	cargo test --all

run-tui:
	cargo run -p tui -- --file docs/example.md

# WASM Commands
build-wasm:
	cd markdown_engine && wasm-pack build --target web --out-dir pkg

# Graphical (Electron/React) Commands
install-graphical:
	cd graphical && npm install

dev-graphical:
	cd graphical && npm run dev

build-graphical:
	cd graphical && npm run build

# Clean up
clean:
	cargo clean
	cd graphical && rm -rf node_modules dist dist-electron
