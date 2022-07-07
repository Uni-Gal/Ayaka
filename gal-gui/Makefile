node_modules: package.json
	npm install

.PHONY: clean
clean:
	rm -rf node_modules
	rm -rf dist

.PHONY: run dist release
run: node_modules
	npm run tauri dev -- -- --no-default-features -- $(FILE)
dist: node_modules
	npm run build
release: node_modules
	STATIC_VCRUNTIME=false npm run tauri build
