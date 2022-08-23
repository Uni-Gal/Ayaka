node_modules: package.json
	npm install

.PHONY: clean
clean:
	rm -rf node_modules
	rm -rf dist

.PHONY: run dist
run: node_modules
	npm run tauri dev -- -- --no-default-features -- $(FILE)
dist: node_modules
	npm run build
