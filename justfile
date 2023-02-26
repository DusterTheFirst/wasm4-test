WASM_FILE := "target/wasm32-unknown-unknown/wasm/cart.wasm"

build:
    cargo build --profile wasm
    # wasm-opt -Oz {{WASM_FILE}} -o {{WASM_FILE}}

size: build
    du -bh {{WASM_FILE}}

run: build size
    w4 run-native {{WASM_FILE}}