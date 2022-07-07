.PHONY: test clean update
test: plugins dist-gui
	cargo test
clean:
	cargo clean
	cd gal-gui && $(MAKE) clean
update:
	cargo update
	cd gal-gui && $(MAKE) node_modules

.PHONY: plugins release release-gui dist-gui
plugins:
	cd plugins && $(MAKE)
release:
	cargo build --manifest-path=gal/Cargo.toml --release
release-gui:
	cd gal-gui && $(MAKE) release
dist-gui:
	cd gal-gui && $(MAKE) dist

EXAMPLES:=Fibonacci Fibonacci2 Orga

define example-tpl
.PHONY: example-$(1) example-$(1)-gui
example-$(1): examples/$(1)/config.yaml plugins
	RUST_LOG=info cargo run --manifest-path=gal/Cargo.toml -- $$< --auto
example-$(1)-gui: examples/$(1)/config.yaml plugins
	cd gal-gui && $$(MAKE) run FILE=$$(realpath $$<)
endef

$(eval $(foreach ex,$(EXAMPLES),$(call example-tpl,$(ex))))

.SECONDARY:
