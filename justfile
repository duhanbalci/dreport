# dreport justfile

# Frontend dev server
front:
    cd frontend && bun run dev

# Backend dev server
back:
    cargo run -p dreport-backend

# Frontend + Backend birlikte
dev:
    just front & just back & wait

# WASM build (core -> frontend)
wasm:
    wasm-pack build core --target web --release --out-dir ../frontend/src/core/wasm-pkg -- --features wasm
    cp frontend/src/core/wasm-pkg/dreport_core.js frontend/src/core/wasm/dreport_core.js
    cp frontend/src/core/wasm-pkg/dreport_core.d.ts frontend/src/core/wasm/dreport_core.d.ts
    cp frontend/src/core/wasm-pkg/dreport_core_bg.wasm frontend/src/core/wasm/dreport_core_bg.wasm
    cp frontend/src/core/wasm-pkg/dreport_core_bg.wasm.d.ts frontend/src/core/wasm/dreport_core_bg.wasm.d.ts
    cp frontend/src/core/wasm/dreport_core_bg.wasm frontend/public/wasm/dreport_core_bg.wasm
