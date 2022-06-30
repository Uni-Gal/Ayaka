node_modules: package.json
	npm install

.PHONY: run release
run: node_modules
	npm run tauri dev -- -- -- $(FILE)
release: node_modules
	npm run tauri build
