use chrono::{DateTime, Utc};
use wspr_live_client::{ReceptionQuery, WsprLiveClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args.len() > 5 {
        eprintln!("Usage: cargo run --example receiving -- <start-UTC> <end-UTC> [callsign] [band]");
        eprintln!("Example: cargo run --example receiving -- 2026-11-17T00:00:00Z 2026-11-18T00:00:00Z n9iox 14");
        std::process::exit(2);
    }

    let start = DateTime::parse_from_rfc3339(&args[1])?.with_timezone(&Utc);
    let end = DateTime::parse_from_rfc3339(&args[2])?.with_timezone(&Utc);
    let callsign = args.get(3).map(String::as_str).unwrap_or("n9iox");
    let mut query = ReceptionQuery::new(callsign, start, end);
    if let Some(band) = args.get(4) {
        query = query.band(band.parse()?);
    }

    let spots = WsprLiveClient::new()?.receptions(&query).await?;
    println!("Received {} spots for {}", spots.len(), callsign.to_ascii_uppercase());
    for spot in spots {
        println!("{}  {:<12}  {:>10} Hz  {:>4} dB  {:>5} km  {}",
            spot.time, spot.tx_sign, spot.frequency, spot.snr, spot.distance, spot.tx_loc);
    }
    Ok(())
}
