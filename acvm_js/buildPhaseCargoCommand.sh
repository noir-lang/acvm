#!/usr/bin/env bash

# Clear out the existing build artifacts as these aren't automatically removed by wasm-pack.
if [ -d ./pkg/ ]; then
    rm -rf ./pkg/
fi

WASM_BINARY=./target/wasm32-unknown-unknown/release/acvm_simulator.wasm
NODE_WASM=./pkg/nodejs/acvm_simulator_bg.wasm
BROWSER_WASM=./pkg/nodejs/acvm_simulator_bg.wasm

# Build the new wasm package
cargo build --lib --release --target wasm32-unknown-unknown
wasm-bindgen $WASM_BINARY --out-dir ./pkg/nodejs --typescript --target nodejs
wasm-bindgen $WASM_BINARY --out-dir ./pkg/web --typescript --target web
wasm-opt $NODE_WASM -o $NODE_WASM -O
wasm-opt $BROWSER_WASM -o $BROWSER_WASM -O