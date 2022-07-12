PROJECT_ROOT:=..

PLUGIN_NAMES:=format slog markdown random
PLUGIN_TARGETS:=$(PLUGIN_NAMES:%=$(PROJECT_ROOT)/target/wasm32-wasi/release/%.wasm)

.PHONY: $(PLUGIN_TARGETS)
plugins: $(PLUGIN_TARGETS)

define plugin-tpl
$(PROJECT_ROOT)/target/wasm32-wasi/release/$(1).wasm:
	cargo build --package $(1) --target=wasm32-wasi --release

endef

$(eval $(foreach p,$(PLUGIN_NAMES),$(call plugin-tpl,$(p))))
