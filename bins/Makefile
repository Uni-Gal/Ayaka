.PHONY: clean update
clean:
	cargo clean
update:
	cargo update

.PHONY: release
release:
	cd gal-gui && $(MAKE) dist
	STATIC_VCRUNTIME=false cargo build --release

.PHONY: run run-gui run-latex
run:
	RUST_LOG=info cargo run --package gal -- $(FILE) --auto
run-gui:
	cd gal-gui && $(MAKE) run FILE=$(FILE)
run-latex:
	RUST_LOG=info cargo run --package gal-latex -- $(FILE) -o $(basename $(FILE)).tex
