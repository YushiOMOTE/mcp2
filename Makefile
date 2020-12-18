crate_name ?= mcp2
web_dir ?= static/
wasm_dir ?= static/target


.PHONY: setup build run tiles


setup:
	rustup target add wasm32-unknown-unknown
	cargo install wasm-bindgen-cli
	cargo install basic-http-server


build: tiles
	cargo build --target wasm32-unknown-unknown --release
	wasm-bindgen --out-dir $(wasm_dir) --target web target/wasm32-unknown-unknown/release/$(crate_name).wasm


tiles:
	cd tiles && cargo run -p tiles -- test-tiled.tmx . ../src/tiles.json


run: build
	basic-http-server $(web_dir)

