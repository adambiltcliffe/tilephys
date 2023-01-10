desktop:
	cargo build && ./target/debug/tilephys
web:
	cargo build --target wasm32-unkown-unknown && cp ./target/wasm32-unknown-unknown/debug/tilephys.wasm ./princess-robot.wasm