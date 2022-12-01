node_modules: package.json
	npm install

.PHONY: clean
clean:
	rm -rf node_modules
	rm -rf dist
	rm -f package-lock.json

.PHONY: run dist
run: node_modules
	npm run tauri dev -- -- -- $(FILE)
dist: node_modules
	npm run build
