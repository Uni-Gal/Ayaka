node_modules: package.json
	npm install

.PHONY: clean
clean:
	rm -rf node_modules
	rm -rf dist

.PHONY: run release
run: node_modules
	npm run tauri dev -- -- -- $(FILE)
release: node_modules
	npm run tauri build
