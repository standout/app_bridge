# Example App

This example app is created to use in the tests.

## Setup

### Setup Rust Using `asdf`

```bash
asdf plugin-add rust
asdf install
```

## Compile

```bash
cargo clean && cargo build --target wasm32-wasip2 --release
```

## Good to know

Verify the inluded WIT by running:

```bash
wasm-tools component wit target/wasm32-wasip2/release/example.wasm
```

It should include the WIT-file defined in `ext/app_bridge/wit/world.wit`.
