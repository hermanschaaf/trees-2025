build:
	cd tree-rs && wasm-pack build --target web
	cp tree-rs/pkg/tree_rs.js web/static/js/
	cp tree-rs/pkg/tree_rs_bg.wasm web/static/js/

serve:
	cd web && python3 -m http.server 8000

build-and-serve: build serve

clean:
	cd tree-rs && wasm-pack clean