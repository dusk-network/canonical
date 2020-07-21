#!/bin/sh
cargo build --target wasm32-unknown-unknown --release &&
cp target/wasm32-unknown-unknown/release/counter.wasm counter.wasm
#printwat counter.wasm
