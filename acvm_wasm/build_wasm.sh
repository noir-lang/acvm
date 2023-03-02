#! /bin/bash

cargo wasi build --target wasm32-unknown-unknown
wasm-tools component new ../target/wasm32-unknown-unknown/debug/acvm_wasm.wasm \
    -o ../target/wasm32-unknown-unknown/debug/acvm.component.wasm --adapt ./wasi_snapshot_preview1.wasm
