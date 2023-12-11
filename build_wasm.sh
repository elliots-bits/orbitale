cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir ./orbitale_release/ --target web ./target/wasm32-unknown-unknown/release/space_chase.wasm
mkdir orbitale_release/assets/
cp -r assets/* orbitale_release/assets/
cp index.html orbitale_release/index.html