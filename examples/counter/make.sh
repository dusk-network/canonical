#!/bin/sh
cargo build --target wasm32-unknown-unknown --release &&
wasm-opt -Oz target/wasm32-unknown-unknown/release/counter.wasm -o counter.wasm
stat counter.wasm
