node_modules: package.json
	npm install

.PHONY: clean
clean:
	rm -rf node_modules
	rm -rf dist
	rm -f package-lock.json

.PHONY: run dist
run: node_modules
	cargo tauri dev -- -- $(FILE)
dist: node_modules
	npm run build

build_android: dist
	cargo tauri android build
	rm -r src-tauri/.cargo

build_ios: dist
	cargo tauri ios build
	rm -r src-tauri/.cargo
