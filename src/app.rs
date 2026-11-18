use chrono::{TimeDelta, Utc};
use gloo_net::http::Request;
use leptos::{html, prelude::*, task::spawn_local};
use wasm_bindgen::prelude::*;
use wspr_live_client::{Reception, SpotDirection};

use crate::search::{DATETIME_FORMAT, SEARCH_PATH, SPOT_LIMIT, SearchRequest};

#[wasm_bindgen(module = "/src/map.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = createMap)]
    fn create_map(target: &str, reports: &str, transmitted: bool) -> Result<u32, JsValue>;
    #[wasm_bindgen(js_name = disposeMap)]
    fn dispose_map(id: u32);
}

#[component]
fn ReceptionMap(reports: String, direction: SpotDirection) -> impl IntoView {
    let target = NodeRef::<html::Div>::new();
    let error = RwSignal::new(None::<String>);
    Effect::new(move |_| {
        if target.get().is_some() {
            match create_map("reception-map", &reports, direction == SpotDirection::Transmitted) {
                Ok(id) => on_cleanup(move || dispose_map(id)),
                Err(_) => error.set(Some("The map could not be loaded. Reception reports are available in the table below.".into())),
            }
        }
    });

    view! {
        <div class="map-legend">
            <span><span class="home-marker" aria-hidden="true">"★"</span>
                {match direction {
                    SpotDirection::Transmitted => " Your transmitter",
                    SpotDirection::Received => " Your receiver",
                }}</span>
            <span><span class="remote-marker" aria-hidden="true">"●"</span>
                {match direction {
                    SpotDirection::Transmitted => " Receiving stations",
                    SpotDirection::Received => " Transmitting stations",
                }}</span>
        </div>
        <p class="map-help">"Markers show grid-square centers. Select a marker for station details."</p>
        <p role="alert">{move || error.get()}</p>
        <div node_ref=target id="reception-map" class="reception-map" tabindex="0"
            role="region" aria-label="Map of receiver and transmitting stations" />
    }
}

const BANDS: [(i16, &str); 17] = [
    (-1, "LF"),
    (0, "MF"),
    (1, "160 m"),
    (3, "80 m"),
    (5, "60 m"),
    (7, "40 m"),
    (10, "30 m"),
    (14, "20 m"),
    (18, "17 m"),
    (21, "15 m"),
    (24, "12 m"),
    (28, "10 m"),
    (50, "6 m"),
    (70, "4 m"),
    (144, "2 m"),
    (432, "70 cm"),
    (1296, "23 cm"),
];

const RECENT_RANGES: [(i64, &str); 13] = [
    (10, "10 min"),
    (30, "30 min"),
    (60, "1 hour"),
    (120, "2 hours"),
    (360, "6 hours"),
    (720, "12 hours"),
    (1440, "24 hours"),
    (2880, "2 days"),
    (4320, "3 days"),
    (5760, "4 days"),
    (7200, "5 days"),
    (8640, "6 days"),
    (10080, "7 days"),
];

fn recent_range(minutes: i64) -> (String, String) {
    let end = Utc::now();
    let start = end - TimeDelta::minutes(minutes);
    (
        start.format(DATETIME_FORMAT).to_string(),
        end.format(DATETIME_FORMAT).to_string(),
    )
}

enum SearchState {
    Idle,
    Searching,
    Finished(Result<(SpotDirection, Vec<Reception>), String>),
}

#[component]
pub fn App() -> impl IntoView {
    let callsign = NodeRef::<html::Input>::new();
    let band = NodeRef::<html::Select>::new();
    let power = NodeRef::<html::Select>::new();
    let state = RwSignal::new(SearchState::Idle);
    let (default_start, default_end) = recent_range(1440);
    let start = RwSignal::new(default_start);
    let end = RwSignal::new(default_end);
    let selected_range = RwSignal::new(Some(1440));
    let direction = RwSignal::new(SpotDirection::Transmitted);
    let searching = move || state.with(|state| matches!(state, SearchState::Searching));
    let set_range = move |minutes| {
        let (new_start, new_end) = recent_range(minutes);
        start.set(new_start);
        end.set(new_end);
        selected_range.set(Some(minutes));
    };

    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if searching() {
            return;
        }
        if let Some(minutes) = selected_range.get_untracked() {
            set_range(minutes);
        }
        let request = SearchRequest {
            callsign: callsign.get().expect("callsign input is mounted").value(),
            start: start.get_untracked(),
            end: end.get_untracked(),
            direction: direction.get_untracked(),
            power: power
                .get()
                .expect("power select is mounted")
                .value()
                .parse()
                .ok(),
            band: band
                .get()
                .expect("band select is mounted")
                .value()
                .parse()
                .ok(),
        };
        state.set(SearchState::Searching);
        spawn_local(async move {
            let direction = request.direction;
            state.set(SearchState::Finished(
                search(request).await.map(|spots| (direction, spots)),
            ));
        });
    };

    view! {
        <main>
            <h1>"WSPRviz"</h1>
            <form aria-label="Search WSPR reception reports" on:submit=submit>
                <fieldset class="range-presets" disabled=searching>
                    <legend>"Last"</legend>
                    <div class="range-buttons">
                        {RECENT_RANGES.into_iter().map(|(minutes, label)| view! {
                            <button type="button"
                                aria-pressed=move || (selected_range.get() == Some(minutes)).to_string()
                                on:click=move |_| set_range(minutes)>{label}</button>
                        }).collect_view()}
                    </div>
                </fieldset>
                <label>
                    "Your callsign"
                    <input node_ref=callsign name="callsign" value="N9IOX" required maxlength="20" spellcheck="false" />
                </label>
                <label>
                    "Show"
                    <select name="direction" disabled=searching
                        on:change=move |event| direction.set(match event_target_value(&event).as_str() {
                            "received" => SpotDirection::Received,
                            _ => SpotDirection::Transmitted,
                        })>
                        <option value="transmitted">"Who received me"</option>
                        <option value="received">"What I received"</option>
                    </select>
                </label>
                <label>
                    "Start (UTC)"
                    <input name="start" type="datetime-local" required prop:value=move || start.get()
                        on:input=move |event| {
                            start.set(event_target_value(&event));
                            selected_range.set(None);
                        } />
                </label>
                <label>
                    "End (UTC, exclusive)"
                    <input name="end" type="datetime-local" required prop:value=move || end.get()
                        on:input=move |event| {
                            end.set(event_target_value(&event));
                            selected_range.set(None);
                        } />
                </label>
                <label>
                    "Band"
                    <select node_ref=band name="band">
                        <option value="">"All bands"</option>
                        {BANDS.into_iter().map(|(value, label)| view! {
                            <option value=value.to_string()>{label}</option>
                        }).collect_view()}
                    </select>
                </label>
                <label>
                    "TX power"
                    <select node_ref=power name="power" disabled=searching>
                        <option value="">"All powers"</option>
                        <option value="27">"500 mW"</option>
                        <option value="30">"1 W"</option>
                    </select>
                </label>
                <button type="submit" disabled=searching>"Search"</button>
            </form>
            <section aria-label="Search results">
                {move || state.with(|state| match state {
                    SearchState::Idle => ().into_any(),
                    SearchState::Searching => view! { <p role="status">"Searching WSPR.live…"</p> }.into_any(),
                    SearchState::Finished(Err(error)) => view! { <p role="alert">{error.clone()}</p> }.into_any(),
                    SearchState::Finished(Ok((_, spots))) if spots.is_empty() => {
                        view! { <p role="status">"No reception reports found."</p> }.into_any()
                    }
                    SearchState::Finished(Ok((direction, spots))) => results(spots, *direction).into_any(),
                })}
            </section>
        </main>
    }
}

async fn search(request: SearchRequest) -> Result<Vec<Reception>, String> {
    let response = Request::post(SEARCH_PATH)
        .json(&request)
        .map_err(|_| "Could not prepare the search request.")?
        .send()
        .await
        .map_err(|_| "Could not reach the search server. Try again.")?;
    if !response.ok() {
        return Err(response
            .text()
            .await
            .unwrap_or_else(|_| "Search failed. Try again.".into()));
    }
    response
        .json()
        .await
        .map_err(|_| "The search server returned an invalid response.".into())
}

fn results(spots: &[Reception], direction: SpotDirection) -> impl IntoView + use<> {
    let count = spots.len();
    let summary = if count == SPOT_LIMIT as usize {
        format!("Showing the newest {count} spots. Narrow the search to see more specific results.")
    } else {
        match direction {
            SpotDirection::Transmitted => format!("{count} reports of your transmissions"),
            SpotDirection::Received => format!("{count} reports you received"),
        }
    };
    let rows = spots
        .iter()
        .map(|spot| {
            view! {
                <tr>
                    <td>{spot.time.clone()}</td>
                    <td>{spot.rx_sign.clone()}</td>
                    <td>{spot.rx_loc.clone()}</td>
                    <td>{spot.tx_sign.clone()}</td>
                    <td>{spot.tx_loc.clone()}</td>
                    <td class="number">{spot.frequency}</td>
                    <td class="number">{format!("{:.3}", 10_f64.powf((f64::from(spot.power) - 30.0) / 10.0))}</td>
                    <td class="number">{spot.snr}</td>
                    <td class="number">{spot.distance}</td>
                </tr>
            }
        })
        .collect_view();
    view! {
        <p role="status">{summary}</p>
        <ReceptionMap reports=serde_json::to_string(spots).expect("reception reports serialize") direction />
        <div class="results-table" tabindex="0" role="region" aria-label="Reception reports table">
            <table>
                <thead><tr>
                    <th scope="col">"Time (UTC)"</th>
                    <th scope="col">"Receiver"</th>
                    <th scope="col">"RX grid"</th>
                    <th scope="col">"Transmitter"</th>
                    <th scope="col">"TX grid"</th>
                    <th scope="col" class="number">"Frequency (Hz)"</th>
                    <th scope="col" class="number">"TX power (W)"</th>
                    <th scope="col" class="number">"SNR (dB)"</th>
                    <th scope="col" class="number">"Distance (km)"</th>
                </tr></thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
}
