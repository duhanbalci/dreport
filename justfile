# dreport justfile

# Frontend dev server
front:
    cd frontend && bun run dev

# Backend dev server
back:
    cargo run -p dreport-backend

# Frontend + Backend + WASM watch birlikte
dev:
    just front & just back & just wasm-watch & wait

# Layout engine WASM build
wasm:
    wasm-pack build layout-engine --target web --release --out-dir ../frontend/src/core/wasm-pkg-layout
    mkdir -p frontend/src/core/wasm-layout frontend/public/wasm
    cp frontend/src/core/wasm-pkg-layout/dreport_layout.js frontend/src/core/wasm-layout/dreport_layout.js
    cp frontend/src/core/wasm-pkg-layout/dreport_layout.d.ts frontend/src/core/wasm-layout/dreport_layout.d.ts
    cp frontend/src/core/wasm-pkg-layout/dreport_layout_bg.wasm frontend/public/wasm/dreport_layout_bg.wasm
    cp frontend/src/core/wasm-pkg-layout/dreport_layout_bg.wasm.d.ts frontend/src/core/wasm-layout/dreport_layout_bg.wasm.d.ts

# Layout engine WASM watch (rebuild on change)
wasm-watch:
    watchexec -w layout-engine/src -w core/src -e rs -- just wasm

# --- Test Komutlari ---

# Rust testleri (core + layout-engine + backend)
test-rust:
    cargo test

# Frontend unit testleri (Vitest)
test-front:
    cd frontend && bun run test:run

# Generate PDF reference PNGs for cross-renderer visual tests
visual-refs:
    cargo test -p dreport-layout --test visual_test -- generate_cross_renderer --ignored

# Rust visual snapshot testleri
test-visual-rust:
    cargo test -p dreport-layout --test visual_test

# Cross-renderer visual testleri (Playwright: HTML vs PDF)
test-visual-cross: visual-refs
    cd frontend && bun run test:visual -- --project=cross-renderer

# Editor visual testleri (Playwright)
test-visual-editor:
    cd frontend && bun run test:visual -- --project=editor

# Tum visual testler (Playwright: editor + cross-renderer)
test-visual: visual-refs
    cd frontend && bun run test:visual

# Tum testler (Rust + frontend unit + visual)
test-all: test-rust test-front test-visual

# Visual diff sonuclarini ac (cross-renderer)
diff-open:
    #!/usr/bin/env bash
    DIFF_DIR="frontend/tests/visual/cross-renderer-diffs"
    if [ -z "$(ls -A "$DIFF_DIR" 2>/dev/null)" ]; then
        echo "Diff klasoru bos — once 'just test-visual-cross' calistirin."
        exit 1
    fi
    open "$DIFF_DIR"/*_diff.png "$DIFF_DIR"/*_html.png 2>/dev/null || xdg-open "$DIFF_DIR" 2>/dev/null || echo "Dosyalar: $DIFF_DIR"

# --- Lint / Format / Build ---

# Rust + frontend lint
lint:
    cargo clippy --workspace -- -D warnings
    cd frontend && bun run lint

# Rust + frontend format
fmt:
    cargo fmt --workspace
    cd frontend && bun run format

# Format kontrolu (CI icin)
fmt-check:
    cargo fmt --workspace --check
    cd frontend && bun run format:check

# Full build
build:
    cd frontend && bun run build
    cargo build --release -p dreport-backend

# Type check (Rust + TypeScript)
check:
    cargo check --workspace
    cd frontend && bun run type-check

# --- Publish ---

# Publish dreport-core to Gitea
publish-core:
    cargo publish -p dreport-core --registry gitea --allow-dirty

# Publish dreport-layout to Gitea (depends on core)
publish-layout:
    cargo publish -p dreport-layout --registry gitea --allow-dirty

# Publish all crates to Gitea (in order)
publish-all:
    just publish-core
    just publish-layout
