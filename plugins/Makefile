.PHONY: test clean update
test:
	cargo test
clean:
	cargo clean
update:
	cargo update

.PHONY: plugins
plugins:
	cargo build --target=wasm32-wasi --release
	mkdir -p ../examples/plugins
	cp target/wasm32-wasi/release/*.wasm ../examples/plugins/
