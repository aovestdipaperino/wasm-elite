#!/usr/bin/env bash
# Build the wasm bundle into web/pkg/. Deploy with:
#   cd web && npx -y netlify-cli deploy --prod --dir=. --no-build --message "..."
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p elite-wasm --release --target wasm32-unknown-unknown
rm -rf web/pkg && mkdir -p web/pkg
wasm-bindgen target/wasm32-unknown-unknown/release/elite_wasm.wasm --target web --no-typescript --out-dir web/pkg
wasm-opt -Oz --enable-reference-types --enable-bulk-memory --enable-nontrapping-float-to-int --enable-multivalue --enable-sign-ext -o web/pkg/elite_wasm_bg.wasm web/pkg/elite_wasm_bg.wasm
ls -la web/pkg
