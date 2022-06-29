.PHONY: test clean update
test: plugins
	cargo test
clean:
	cargo clean
update:
	cargo update

.PHONY: plugins
plugins:
	cd plugins && $(MAKE)

.PHONY: sample sample-release
sample: sample.yaml plugins
	RUST_LOG=info cargo run --manifest-path=gal/Cargo.toml -- $< --auto

sample-release: sample.yaml plugins
	cargo run --manifest-path=gal/Cargo.toml --release -- $< --auto

.SECONDARY:
