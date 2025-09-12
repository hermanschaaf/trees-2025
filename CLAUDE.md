# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a 3D tree generation project that combines Rust-based procedural tree generation with Three.js visualization. The Rust code generates tree structures (using a novel ring-based architecture) and compiles to WebAssembly for use in a web-based 3D viewer.

## Architecture

### Core Components

- **tree-rs/**: Rust crate that generates 3D tree structures
  - `tree.rs`: Core tree generation logic using ring-based architecture (TreeRing, TreeStructure, TreeSpecies)
  - `lib.rs`: WASM bindings and exports for web integration
  - Compiles to WebAssembly for web usage

- **web/**: Three.js frontend for 3D visualization
  - `main.ts`: Three.js scene setup and tree rendering
  - Imports generated WASM module for tree data
  - Uses OrbitControls and dat.gui for interaction

### Tree Generation Architecture

The project uses a unique "ring-based" approach where trees are composed of TreeRing structures rather than traditional branches. Each ring has:
- Geometric properties (center, radius, orientation)
- Tree structure data (growth_factor, bark_thickness)
- Connectivity (parent/child relationships)

## Development Commands

### Building the Project
```bash
make build  # Builds Rust to WASM and copies files to web/static/js/
```

### Development Server
```bash
make serve  # Starts Vite dev server in web/ directory
```

### Combined Build and Serve
```bash
make build-and-serve  # Builds then serves
```

### Cleaning Build Artifacts
```bash
make clean  # Cleans wasm-pack build artifacts
```

### Individual Component Commands

**Rust/WASM Build:**
```bash
cd tree-rs && RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --target web
```

**Web Development:**
```bash
cd web && npx vite        # Dev server
cd web && npx vite build  # Production build
```

## Key Files and Dependencies

- `tree-rs/Cargo.toml`: Rust dependencies including wasm-bindgen, glam, micromath, rand
- `web/package.json`: Frontend dependencies (Three.js, TypeScript, Vite, dat.gui)
- `Makefile`: Build automation and development workflow
- Generated WASM files are copied to `web/static/js/` during build process

## Development Workflow

1. Modify tree generation logic in `tree-rs/src/`
2. Run `make build` to compile Rust to WASM
3. Run `make serve` to start development server
4. The web app at `web/main.ts` will use the updated WASM module