# WSPRviz

Minimal client-side Leptos 0.8 starter with a standalone Rust client for [WSPR.live](https://wspr.live/).

## Run the web starter

Install Rust from https://rustup.rs/, then:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve --open
```

Build: `trunk build --release`. Static output is in `dist/`.

## Query WSPR.live

The separate `wspr-live-client/` crate queries WSPR reception reports for receiver callsign N9IOX over a UTC time range. See [its README](wspr-live-client/README.md).

```sh
cd wspr-live-client
cargo run --example receiving -- 2026-11-17T00:00:00Z 2026-11-18T00:00:00Z N9IOX 14
```

The Leptos UI does not yet invoke the native HTTP client. A Rust backend can expose it to the browser later.
