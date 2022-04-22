#!/bin/bash
RUSTFLAGS="-C opt-level=3" wasm-pack build --profiling --out-name indexer --out-dir ./builds/indexer -- --features indexing 