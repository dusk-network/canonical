NAME=counter

all: opt

release:
	cargo build --target wasm32-unknown-unknown --release

opt: release
	wasm-opt --strip --strip-producers -Oz ../../../target/wasm32-unknown-unknown/release/$(NAME).wasm -o $(NAME).wasm
