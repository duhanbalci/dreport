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

# Eski core WASM build (typst-based, deprecated)
wasm-legacy:
    wasm-pack build core --target web --release --out-dir ../frontend/src/core/wasm-pkg -- --features wasm
    cp frontend/src/core/wasm-pkg/dreport_core.js frontend/src/core/wasm/dreport_core.js
    cp frontend/src/core/wasm-pkg/dreport_core.d.ts frontend/src/core/wasm/dreport_core.d.ts
    cp frontend/src/core/wasm-pkg/dreport_core_bg.wasm frontend/src/core/wasm/dreport_core_bg.wasm
    cp frontend/src/core/wasm-pkg/dreport_core_bg.wasm.d.ts frontend/src/core/wasm/dreport_core_bg.wasm.d.ts
    cp frontend/src/core/wasm/dreport_core_bg.wasm frontend/public/wasm/dreport_core_bg.wasm
