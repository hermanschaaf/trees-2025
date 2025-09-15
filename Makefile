build:
	cd packages/tree-engine && RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --target web
	mkdir -p packages/web/public/static/js packages/web/public/static/twigs packages/web/public/static/textures/bark
	cp packages/tree-engine/pkg/tree_rs.js packages/web/public/static/js/
	cp packages/tree-engine/pkg/tree_rs_bg.wasm packages/web/public/static/js/
	cp -r assets/twigs/* packages/web/public/static/twigs/
	cp -r assets/textures/bark/* packages/web/public/static/textures/bark/
	npm run build

serve:
	npm run dev

dev:
	npm run dev

build-and-serve: build serve

clean:
	npm run clean

install:
	npm run install:all