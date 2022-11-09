
#!/bin/bash
set -eu

rustup target add wasm32-unknown-unknown
cargo install -f wasm-bindgen-cli --version 0.2.81
cargo install basic-http-server
