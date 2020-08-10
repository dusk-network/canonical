#!/bin/sh
cargo build --target wasm32-unknown-unknown --release --color=always 2>&1 &&
wasm-opt -Oz target/wasm32-unknown-unknown/release/counter.wasm -o counter.wasm
stat counter.wasm | head -2
