use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use chrono::{DateTime, NaiveDateTime, Utc};
use tower_http::services::ServeDir;
use wspr_live_client::{Error, Reception, ReceptionQuery, WsprLiveClient};

use crate::search::{DATETIME_FORMAT, SEARCH_PATH, SPOT_LIMIT, SearchRequest};

pub async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    if !std::path::Path::new("dist/index.html").is_file() {
        return Err(
            "Build the frontend with `trunk build` from the project directory first.".into(),
        );
    }

    let app = Router::new()
        .route(SEARCH_PATH, post(search_receptions))
        .with_state(WsprLiveClient::new()?)
        .fallback_service(ServeDir::new("dist"));
    let address = std::env::var("WSPRVIZ_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("WSPRviz is listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn search_receptions(
    State(client): State<WsprLiveClient>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<Vec<Reception>>, (StatusCode, String)> {
    let start = parse_utc(&request.start, "Start")?;
    let end = parse_utc(&request.end, "End")?;
    let mut query = ReceptionQuery::new(request.callsign, start, end)
        .direction(request.direction)
        .limit(SPOT_LIMIT);
    if let Some(band) = request.band {
        query = query.band(band);
    }
    if let Some(power) = request.power {
        query = query.power(power);
    }

    client
        .receptions(&query)
        .await
        .map(Json)
        .map_err(|error| match error {
            Error::Http(error) => {
                eprintln!("WSPR.live request failed: {}", error.without_url());
                (
                    StatusCode::BAD_GATEWAY,
                    "WSPR.live is unavailable or returned an invalid response. Try again shortly."
                        .into(),
                )
            }
            error => (StatusCode::BAD_REQUEST, error.to_string()),
        })
}

fn parse_utc(value: &str, field: &str) -> Result<DateTime<Utc>, (StatusCode, String)> {
    NaiveDateTime::parse_from_str(value, DATETIME_FORMAT)
        .map(|time| time.and_utc())
        // ClickHouse DateTime stores an unsigned 32-bit Unix timestamp.
        .ok()
        .filter(|time| u32::try_from(time.timestamp()).is_ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!("Enter a valid {field} date and time in UTC."),
            )
        })
}
