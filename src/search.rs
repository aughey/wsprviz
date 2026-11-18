use serde::{Deserialize, Serialize};
use wspr_live_client::SpotDirection;

pub const SEARCH_PATH: &str = "/api/receptions";
pub const SPOT_LIMIT: u32 = 500;
pub const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M";

#[derive(Deserialize, Serialize)]
pub struct SearchRequest {
    pub callsign: String,
    pub start: String,
    pub end: String,
    pub band: Option<i16>,
    pub power: Option<i8>,
    #[serde(default)]
    pub direction: SpotDirection,
}
