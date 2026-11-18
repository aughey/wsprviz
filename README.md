# WSPRviz

Search [WSPR.live](https://wspr.live/) reports by callsign, UTC time range, and optional band. The default “Who received me” mode finds stations that heard your transmissions. Select “What I received” to find transmissions decoded by your receiver. The Leptos frontend submits the filters to the Rust server, which queries WSPR.live and returns up to 500 spots, newest first.

Use the TX power filter to compare 500 mW and 1 W within the selected time range. These match the message's encoded power of 27 dBm and 30 dBm respectively. “All powers” includes every reported power. Filtering runs on the server before the 500-spot limit, and applies to both the map and table.

Search results include an OpenLayers map with OpenStreetMap tiles. Green stars identify your station and orange dots identify the other stations at the centers of their Maidenhead grid squares. The legend identifies their transmitter or receiver roles for the selected mode. Select a marker to see callsigns and grids. Repeated reports for the same station and grid share a marker. Invalid grid locations remain visible in the results table and are skipped on the map. OpenLayers 10.7.0 is bundled in `vendor/openlayers`, with its license. Map tiles require internet access.

## Run the app

Install Rust from https://rustup.rs/, then:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk build
cargo run
```

Open http://127.0.0.1:3000. The form starts with `N9IOX` and the latest 24 hours in UTC. The start is inclusive and the end is exclusive.

Run these commands from the project directory so the server can serve `dist/`. Use `trunk watch` in a second terminal to rebuild frontend changes, and set `WSPRVIZ_LISTEN_ADDR` to change the server's listen address. For a release build, use `trunk build --release` and `cargo build --release`.

## Query WSPR.live

The standalone Rust client can also run searches from the command line. See the [client README](wspr-live-client/README.md).

```sh
cd wspr-live-client
cargo run --example receiving -- 2026-11-17T00:00:00Z 2026-11-18T00:00:00Z N9IOX 14
```
