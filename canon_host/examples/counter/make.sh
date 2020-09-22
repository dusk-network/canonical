#!/bin/sh
xargo build --target wasm32-unknown-unknown --release --features hosted --color=always 2>&1 &&

wasm-opt --strip --strip-producers -Oz ../../../target/wasm32-unknown-unknown/release/counter.wasm -o counter.wasm &&
		
stat counter.wasm | head -2
