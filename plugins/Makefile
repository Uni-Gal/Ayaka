PROJECT_ROOT:=..

PLUGIN_NAMES:=format random
PLUGIN_TARGETS:=$(PLUGIN_NAMES:%=$(PROJECT_ROOT)/target/wasm32-wasi/release/%.wasm)

.PHONY: $(PLUGIN_TARGETS)
plugins: $(PLUGIN_TARGETS)

define plugin-tpl
$(PROJECT_ROOT)/target/wasm32-wasi/release/$(1).wasm:
	cargo build --manifest-path=$(1)/Cargo.toml --target=wasm32-wasi --release

endef

$(eval $(foreach p,$(PLUGIN_NAMES),$(call plugin-tpl,$(p))))
