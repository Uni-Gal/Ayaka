.PHONY: test clean update
test: plugins
	cargo test
clean:
	cargo clean
	cd gal-gui && $(MAKE) clean
update:
	cargo update
	cd gal-gui && $(MAKE) node_modules

.PHONY: plugins
plugins:
	cd plugins && $(MAKE)

.PHONY: sample sample-gui
sample: sample.yaml plugins
	RUST_LOG=info cargo run --manifest-path=gal/Cargo.toml -- $< --auto
sample-gui: sample.yaml plugins
	cd gal-gui && $(MAKE) run FILE=$(realpath $<)

.PHONY: release release-gui
release:
	cargo build --manifest-path=gal/Cargo.toml --release
release-gui:
	cd gal-gui && $(MAKE) release

.SECONDARY:
