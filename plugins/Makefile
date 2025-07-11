PLUGIN_TARGET:=wasm32-unknown-unknown

.PHONY: test clean update
clean:
	cargo clean
update:
	cargo update --verbose

.PHONY: plugins
plugins:
	cargo build --target $(PLUGIN_TARGET) --release
	mkdir -p ../examples/plugins
	cp target/$(PLUGIN_TARGET)/release/*.wasm ../examples/plugins/
