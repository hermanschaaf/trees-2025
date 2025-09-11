build:
	cd tree-rs && RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --target web
	cp tree-rs/pkg/tree_rs.js web/static/js/
	cp tree-rs/pkg/tree_rs_bg.wasm web/static/js/

serve:
	cd web && npx vite

build-and-serve: build serve

clean:
	cd tree-rs && wasm-pack clean