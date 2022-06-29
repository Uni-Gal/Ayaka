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

.PHONY: front
front:
	cd gal-webfront && $(MAKE) dist

.PHONY: sample-web sample-web-release
sample-web: sample.yaml plugins front
	RUST_LOG=info cargo run --manifest-path=gal-web/Cargo.toml -- $< -p 3000 --dist gal-webfront/dist

sample-web-release: sample.yaml plugins front
	RUST_LOG=info cargo run --manifest-path=gal-web/Cargo.toml --release -- $< -p 3000 --dist gal-webfront/dist

.SECONDARY:
