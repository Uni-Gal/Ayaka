.PHONY: clean update
clean:
	cargo clean
	cd ayaka-gui && $(MAKE) clean
update:
	cargo update --verbose

.PHONY: dist debug-cross release release-cross release-android
dist:
	cd ayaka-gui && $(MAKE) dist
debug-cross: dist
	cargo build --target=$(TARGET)
release: dist
	cargo build --release
release-cross: dist
	cargo build --release --target=$(TARGET)
release-android:
	cd ayaka-gui && $(MAKE) build-android

.PHONY: run run-gui run-latex
run:
	cargo run --package ayaka-check -- $(FILE) --auto
run-gui:
	cd ayaka-gui && $(MAKE) run FILE='$(FILE)'
run-latex:
	cargo run --package ayaka-latex -- $(FILE) -o $(TEXOUT)

.PHONY: run-android run-ios
run-android:
	cd ayaka-gui && $(MAKE) run-android
run-ios:
	cd ayaka-gui && $(MAKE) run-ios
