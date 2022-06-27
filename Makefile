PLUGIN_NAMES:=format
PLUGIN_TARGETS:=$(PLUGIN_NAMES:%=target/wasm32-unknown-unknown/release/%.wasm)

.PHONY: test clean update
test:
	cargo test
clean:
	cargo clean
update:
	cargo update

.PHONY: $(PLUGIN_TARGETS)
plugins: $(PLUGIN_TARGETS)

define plugin-tpl
target/wasm32-unknown-unknown/release/$(1).wasm:
	cargo build --manifest-path=plugins/$(1)/Cargo.toml --target=wasm32-unknown-unknown --release
endef

$(eval $(foreach p,$(PLUGIN_NAMES),$(call plugin-tpl,$(p))))

.PHONY: sample sample-release
sample: sample.yaml plugins
	RUST_LOG=info cargo run --manifest-path=gal/Cargo.toml -- $< --auto

sample-release: sample.yaml plugins
	cargo run --manifest-path=gal/Cargo.toml --release -- $< --auto

.SECONDARY:
