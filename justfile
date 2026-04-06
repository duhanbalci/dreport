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

# Generate PDF reference PNGs for cross-renderer visual tests
visual-refs:
    cargo test -p dreport-layout --test visual_test -- generate_cross_renderer --ignored

# Run cross-renderer visual tests (Playwright vs PDF)
visual-test: visual-refs
    cd frontend && bun run test:visual -- --project=cross-renderer

# Run all visual tests (editor + cross-renderer)
visual-test-all: visual-refs
    cd frontend && bun run test:visual

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
