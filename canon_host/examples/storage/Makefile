NAME=storage

all: opt

release:
	cargo build --target wasm32-unknown-unknown --release --features hosted

opt: release
	wasm-opt --strip --strip-producers -Oz ../../../target/wasm32-unknown-unknown/release/$(NAME).wasm -o $(NAME).wasm
