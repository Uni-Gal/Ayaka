node_modules: package.json
	npm install

.PHONY: run release
run: $(FILE) node_modules
	npm run tauri dev -- -- -- $<
release:
	npm run tauri build
