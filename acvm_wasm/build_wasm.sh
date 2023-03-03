#! /bin/bash

cargo wasi build --target wasm32-unknown-unknown --release
wasm-tools component new ../target/wasm32-unknown-unknown/release/acvm_wasm.wasm \
    -o ../target/wasm32-unknown-unknown/release/acvm.component.wasm --adapt ./wasi_snapshot_preview1.wasm
