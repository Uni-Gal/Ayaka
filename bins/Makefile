.PHONY: clean update
clean:
	cargo clean
	cd ayaka-gui && $(MAKE) clean
update:
	cargo update

.PHONY: dist release release-cross
dist:
	cd ayaka-gui && $(MAKE) dist
release: dist
	STATIC_VCRUNTIME=false cargo build --release
release-cross: dist
	STATIC_VCRUNTIME=false PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target=$(TARGET)

.PHONY: run run-gui run-latex
run:
	RUST_LOG=info cargo run --package ayaka-check -- $(FILE) --auto
run-gui:
	cd ayaka-gui && $(MAKE) run FILE=$(FILE)
run-latex:
	RUST_LOG=info cargo run --package ayaka-latex -- $(FILE) -o $(basename $(FILE)).tex
