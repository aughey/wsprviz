# wspr-live-client

Minimal standalone asynchronous Rust library for querying WSPR.live receiving reports.
It does not depend on Leptos. The existing Leptos starter remains unchanged.

## Run an example

From this directory (Rust and Cargo required):

```sh
cargo run --example receiving -- 2026-11-17T00:00:00Z 2026-11-18T00:00:00Z
```

This defaults to receiver callsign `N9IOX`. Add an explicit callsign and band (14 = 20 meters):

```sh
cargo run --example receiving -- 2026-11-17T00:00:00Z 2026-11-18T00:00:00Z N9IOX 14
```

## Library API

```rust
use chrono::{DateTime, Utc};
use wspr_live_client::{ReceptionQuery, WsprLiveClient};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let start = DateTime::parse_from_rfc3339("2026-11-17T00:00:00Z")?.with_timezone(&Utc);
let end = DateTime::parse_from_rfc3339("2026-11-18T00:00:00Z")?.with_timezone(&Utc);
let query = ReceptionQuery::new("n9iox", start, end).band(14).limit(500);
let spots = WsprLiveClient::new()?.receptions(&query).await?;
# Ok(())
# }
```

`start` is inclusive and `end` exclusive. Dates must be specified with timezone offsets;
all comparisons are performed using UTC epoch seconds. The default result limit is 500;
maximum is 5,000. Results are sorted newest first.

## Data notes

- `rx_sign` identifies the receiving station; `tx_sign` the transmitting station.
- Queries go to `https://db1.wspr.live/?query=...` via HTTPS GET.
- WSPR.live recommends limiting queries by time **and band** when practical. Keep ranges small.
- Raw spots may include duplicates and incorrect reports.
- The public service is for noncommercial use and may be intermittently unavailable.
- Browser-based Leptos/WASM integration would require a browser-compatible HTTP client; this native Rust library is suitable for server-side integration or command-line use without changes. It is deliberately independent of the UI.

See https://wspr.live/ for public API details and service terms.
