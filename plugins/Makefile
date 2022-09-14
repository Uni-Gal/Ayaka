.PHONY: test clean update
clean:
	cargo clean
update:
	cargo update

.PHONY: plugins
plugins:
	cargo build --release
	mkdir -p ../examples/plugins
	cp target/wasm32-wasi/release/*.wasm ../examples/plugins/
	cp target/wasm32-unknown-unknown/release/*.wasm ../examples/plugins/
