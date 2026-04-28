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

# Visual snapshot'lari guncelle (UI degisikliklerinden sonra)
update-snapshots: visual-refs
    cd frontend && bun run test:visual -- --update-snapshots

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

# --- NuGet (Dreport.Service) ---

# Gitea NuGet feed (override env var DREPORT_NUGET_TOKEN if rotated).
NUGET_REGISTRY_URL := "https://gitea.duhanbalci.com/api/packages/duhanbalci/nuget/index.json"
NUGET_TOKEN := env_var_or_default("DREPORT_NUGET_TOKEN", "56b178d79d9cf9dea1c4b90d836d55e41ddff897")
NUGET_VERSION := "0.2.0"

# Build dreport-ffi for the host RID and copy the dylib into runtimes/.
nuget-build-native-host:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --release -p dreport-ffi
    case "$(uname -s)-$(uname -m)" in
      Darwin-arm64)  RID=osx-arm64;   PREFIX=lib; EXT=dylib ;;
      Darwin-x86_64) RID=osx-x64;     PREFIX=lib; EXT=dylib ;;
      Linux-x86_64)  RID=linux-x64;   PREFIX=lib; EXT=so ;;
      Linux-aarch64) RID=linux-arm64; PREFIX=lib; EXT=so ;;
      *) echo "unsupported host: $(uname -s)-$(uname -m)" >&2; exit 1 ;;
    esac
    DEST="bindings/dotnet/src/Dreport.Service/runtimes/$RID/native"
    mkdir -p "$DEST"
    cp "target/release/${PREFIX}dreport_ffi.$EXT" "$DEST/${PREFIX}dreport_ffi.$EXT"

# Cross-compile dreport-ffi for all supported RIDs into runtimes/.
# Requires: rustup targets installed + cargo-zigbuild (`cargo install cargo-zigbuild` and `brew install zig`).
nuget-build-native-all:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v cargo-zigbuild >/dev/null; then
      echo "cargo-zigbuild not found. install with: cargo install cargo-zigbuild && brew install zig" >&2
      exit 1
    fi
    BASE="bindings/dotnet/src/Dreport.Service/runtimes"

    build_target() {
      local TARGET=$1 RID=$2 PREFIX=$3 EXT=$4
      rustup target add "$TARGET" >/dev/null
      cargo zigbuild --release -p dreport-ffi --target "$TARGET"
      mkdir -p "$BASE/$RID/native"
      cp "target/$TARGET/release/${PREFIX}dreport_ffi.$EXT" \
         "$BASE/$RID/native/${PREFIX}dreport_ffi.$EXT"
      echo "✓ $RID"
    }

    build_target aarch64-apple-darwin       osx-arm64   lib dylib
    build_target x86_64-apple-darwin        osx-x64     lib dylib
    build_target x86_64-unknown-linux-gnu   linux-x64   lib so
    build_target aarch64-unknown-linux-gnu  linux-arm64 lib so
    build_target x86_64-pc-windows-gnu      win-x64     ""  dll

# Generate a nuspec for whatever RIDs currently sit in runtimes/, then pack
# the Dreport.Service NuGet package.
nuget-pack:
    #!/usr/bin/env bash
    set -euo pipefail
    PROJ_DIR="bindings/dotnet/src/Dreport.Service"
    NUSPEC_NAME=".generated.nuspec"
    NUSPEC="$PROJ_DIR/$NUSPEC_NAME"
    OUT_DIR="$(pwd)/target/nuget"
    mkdir -p "$OUT_DIR"

    dotnet build "$PROJ_DIR/Dreport.Service.csproj" -c Release --nologo

    # <files> entries for every native binary that exists on disk.
    FILES=""
    while IFS= read -r path; do
      rel="${path#${PROJ_DIR}/}"
      FILES+="    <file src=\"$rel\" target=\"$rel\" />"$'\n'
    done < <(find "$PROJ_DIR/runtimes" -type f \( -name '*.dylib' -o -name '*.so' -o -name '*.dll' \) 2>/dev/null | sort)

    {
      echo '<?xml version="1.0" encoding="utf-8"?>'
      echo '<package xmlns="http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd">'
      echo '  <metadata>'
      echo '    <id>Dreport.Service</id>'
      echo "    <version>{{NUGET_VERSION}}</version>"
      echo '    <authors>dreport</authors>'
      echo '    <description>Native layout engine + PDF renderer for dreport templates. Wraps the dreport-ffi C ABI.</description>'
      echo '    <tags>pdf layout template rendering</tags>'
      echo '    <dependencies><group targetFramework="net8.0" /></dependencies>'
      echo '  </metadata>'
      echo '  <files>'
      echo '    <file src="bin/Release/net8.0/Dreport.Service.dll" target="lib/net8.0/Dreport.Service.dll" />'
      printf '%s' "$FILES"
      echo '  </files>'
      echo '</package>'
    } > "$NUSPEC"

    rm -f "$OUT_DIR/Dreport.Service."*.nupkg
    dotnet pack "$PROJ_DIR/Dreport.Service.csproj" \
      -c Release --no-build --nologo \
      -p:NuspecFile="$NUSPEC_NAME" \
      -p:NuspecBasePath="." \
      -p:IsPackable=true \
      -o "$OUT_DIR"

    echo "package -> $OUT_DIR/Dreport.Service.{{NUGET_VERSION}}.nupkg"
    unzip -l "$OUT_DIR/Dreport.Service.{{NUGET_VERSION}}.nupkg"

# Push the packed nupkg to Gitea.
nuget-push:
    dotnet nuget push \
      "target/nuget/Dreport.Service.{{NUGET_VERSION}}.nupkg" \
      --source "{{NUGET_REGISTRY_URL}}" \
      --api-key "{{NUGET_TOKEN}}" \
      --skip-duplicate

# Pack Dreport.AspNetCore (depends on Dreport.Service via NuGet dependency).
nuget-pack-aspnetcore:
    #!/usr/bin/env bash
    set -euo pipefail
    PROJ_DIR="bindings/dotnet/src/Dreport.AspNetCore"
    NUSPEC_NAME=".generated.nuspec"
    NUSPEC="$PROJ_DIR/$NUSPEC_NAME"
    OUT_DIR="$(pwd)/target/nuget"
    mkdir -p "$OUT_DIR"

    dotnet build "$PROJ_DIR/Dreport.AspNetCore.csproj" -c Release --nologo

    {
      echo '<?xml version="1.0" encoding="utf-8"?>'
      echo '<package xmlns="http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd">'
      echo '  <metadata>'
      echo '    <id>Dreport.AspNetCore</id>'
      echo "    <version>{{NUGET_VERSION}}</version>"
      echo '    <authors>dreport</authors>'
      echo '    <description>ASP.NET Core integration for Dreport.Service: DI registration plus optional /api endpoint mapping.</description>'
      echo '    <tags>pdf layout aspnetcore dreport</tags>'
      echo '    <dependencies>'
      echo '      <group targetFramework="net8.0">'
      echo "        <dependency id=\"Dreport.Service\" version=\"{{NUGET_VERSION}}\" />"
      echo '      </group>'
      echo '    </dependencies>'
      echo '    <frameworkReferences>'
      echo '      <group targetFramework="net8.0">'
      echo '        <frameworkReference name="Microsoft.AspNetCore.App" />'
      echo '      </group>'
      echo '    </frameworkReferences>'
      echo '  </metadata>'
      echo '  <files>'
      echo '    <file src="bin/Release/net8.0/Dreport.AspNetCore.dll" target="lib/net8.0/Dreport.AspNetCore.dll" />'
      echo '  </files>'
      echo '</package>'
    } > "$NUSPEC"

    rm -f "$OUT_DIR/Dreport.AspNetCore."*.nupkg
    dotnet pack "$PROJ_DIR/Dreport.AspNetCore.csproj" \
      -c Release --no-build --nologo \
      -p:NuspecFile="$NUSPEC_NAME" \
      -p:NuspecBasePath="." \
      -p:IsPackable=true \
      -o "$OUT_DIR"

    echo "package -> $OUT_DIR/Dreport.AspNetCore.{{NUGET_VERSION}}.nupkg"
    unzip -l "$OUT_DIR/Dreport.AspNetCore.{{NUGET_VERSION}}.nupkg"

# Push Dreport.AspNetCore to Gitea.
nuget-push-aspnetcore:
    dotnet nuget push \
      "target/nuget/Dreport.AspNetCore.{{NUGET_VERSION}}.nupkg" \
      --source "{{NUGET_REGISTRY_URL}}" \
      --api-key "{{NUGET_TOKEN}}" \
      --skip-duplicate

# Single-host publish (host RID only — fastest, good for dev iterations).
nuget-publish: nuget-build-native-host nuget-pack nuget-push nuget-pack-aspnetcore nuget-push-aspnetcore

# Multi-RID publish (osx-arm64 + osx-x64 + linux-x64 + linux-arm64 + win-x64).
# Requires cargo-zigbuild. Single command, all platforms + AspNetCore, push to Gitea.
nuget-publish-all: nuget-build-native-all nuget-pack nuget-push nuget-pack-aspnetcore nuget-push-aspnetcore
