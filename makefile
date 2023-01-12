desktop:
	cargo build && ./target/debug/princess-robot
web:
	cargo build --target wasm32-unkown-unknown && cp ./target/wasm32-unknown-unknown/debug/princess-robot.wasm .