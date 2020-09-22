#!/bin/sh
xargo build --target wasm32-unknown-unknown --release --features hosted --color=always 2>&1 &&

wasm-opt --strip --strip-producers -Oz ../../../target/wasm32-unknown-unknown/release/storage.wasm -o storage.wasm &&
		
stat storage.wasm | head -2
