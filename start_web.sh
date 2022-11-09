
#!/bin/bash
set -eu

cargo build --release -p headlines --lib --target wasm32-unknown-unknown

wasm-bindgen target/wasm32-unknown-unknown/release/headlines.wasm \
--out-dir webapp --no-modules --no-typescript

cd webapp
sleep 1s && xdg-open http://127.0.0.1:3000 &
basic-http-server --addr 127.0.0.1:3000 .
