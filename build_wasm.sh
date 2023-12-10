cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/space_chase.wasm
cp -r assets out/assets
cp index.html out/index.html