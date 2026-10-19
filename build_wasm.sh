#!/usr/bin/env bash
set -e
echo "Building QuadFurn CAD WebAssembly (WASM) Release..."
trunk build web/index.html --release --dist docs/app --public-url ./
echo "QuadFurn WebAssembly build successful in docs/app/"
