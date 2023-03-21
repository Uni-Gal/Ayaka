.PHONY: clean update
clean:
	cargo clean
	cd ayaka-gui && $(MAKE) clean
update:
	cargo update

.PHONY: dist debug-cross release release-cross
dist:
	cd ayaka-gui && $(MAKE) dist
debug-cross: dist
	cargo build --target=$(TARGET)
release: dist
	cargo build --release
release-cross: dist
	cargo build --release --target=$(TARGET)

.PHONY: run run-gui run-latex
run:
	cargo run --package ayaka-check -- $(FILE) --auto
run-gui:
	cd ayaka-gui && $(MAKE) run FILE='$(FILE)'
run-latex:
	cargo run --package ayaka-latex -- $(FILE) -o $(TEXOUT)
