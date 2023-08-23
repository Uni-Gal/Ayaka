node_modules: package.json
	npm install

.PHONY: clean
clean:
	rm -rf node_modules
	rm -rf dist

.PHONY: run dist
run: node_modules
	cargo tauri dev -- -- $(FILE)
dist: node_modules
	npm run build

build-android: dist
	cargo tauri android build
	rm -r src-tauri/.cargo

build-ios: dist
	cargo tauri ios build
	rm -r src-tauri/.cargo

run-android: node_modules
	cargo tauri android dev
	rm -r src-tauri/.cargo

run-ios: node_modules
	cargo tauri ios dev
	rm -r src-tauri/.cargo
