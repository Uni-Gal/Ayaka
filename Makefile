.PHONY: test clean update
test: plugins
	cargo test --no-default-features
clean:
	cargo clean
	cd gal-gui && $(MAKE) clean
update:
	cargo update
	cd gal-gui && $(MAKE) node_modules

.PHONY: plugins release release-gui
plugins:
	cd plugins && $(MAKE) plugins
release:
	cargo build --package gal --release
release-gui:
	cd gal-gui && $(MAKE) release

EXAMPLES:=Fibonacci Fibonacci2 Gacha Markdown Orga

define example-tpl
.PHONY: example-$(1) example-$(1)-gui example-$(1)-release example-$(1)-gui-release
example-$(1): examples/$(1)/config.yaml plugins
	RUST_LOG=info cargo run --package gal -- $$< --auto
example-$(1)-gui: examples/$(1)/config.yaml plugins
	cd gal-gui && $$(MAKE) run FILE=$$(realpath $$<)
example-$(1)-release: examples/$(1)/config.yaml plugins release
	target/release/gal $$< --auto
example-$(1)-gui-release: examples/$(1)/config.yaml plugins release-gui
	target/release/gal-gui $$<

endef

$(eval $(foreach ex,$(EXAMPLES),$(call example-tpl,$(ex))))

.SECONDARY:
