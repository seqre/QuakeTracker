use std::collections::HashMap;

use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use {reqwest, thiserror};

use crate::seismic::SeismicEvent;

static SEISMIC_URL: &str = "https://www.seismicportal.eu/fdsnws/event/1/query";
static SEISMIC_WSS_URL: &str = "wss://www.seismicportal.eu/standing_order/websocket";

#[derive(Debug, Serialize, thiserror::Error)]
pub enum Error {
    #[error("provided query is incorrect: `{0}`")]
    Validation(String),
    #[error("network connection error: `{0}`")]
    Network(String),
    #[error("could not deserialize response: `{0}`")]
    Deserialization(String),
}

#[tauri::command]
pub async fn get_seismic_events(query_params: QueryParams) -> Result<tauri::ipc::Response, Error> {
    query_params.validate()?;

    let response = reqwest::Client::new()
        .get(SEISMIC_URL)
        .query(&query_params)
        .send()
        .await
        .map_err(|e| Error::Network(e.to_string()))?;

    let events = response
        .text()
        .await
        .map_err(|e| Error::Network(e.to_string()))?;

    Ok(tauri::ipc::Response::new(events))
}

// https://www.seismicportal.eu/realtime.html
#[tauri::command]
pub async fn listen_to_seismic_events(on_event: Channel<String>) {
    let request = SEISMIC_WSS_URL.into_client_request().unwrap();

    let (mut stream, response) = connect_async(request).await.unwrap();

    while let Some(msg) = stream.next().await {
        if let Ok(Message::Text(text)) = msg {
            let s = text.as_str().to_string();
            log::trace!("WSS Message: {s}");

            on_event.send(s).unwrap()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WssAction {
    Create,
    Update,
}

#[derive(Debug, Deserialize)]
struct InnerWssEvent {
    pub action: WssAction,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WssEvent {
    pub action: WssAction,
    pub data: SeismicEvent,
}

// Generated from: https://www.seismicportal.eu/fdsn-wsevent.html
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeConstraints {
    /// The start time of the query, in UTC format
    #[serde(rename = "start", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,
    /// The end time of the query, in UTC format
    #[serde(rename = "end", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoxAreaConstraints {
    /// The minimum latitude of the bounding box, in degrees
    #[serde(rename = "minlat", skip_serializing_if = "Option::is_none")]
    pub min_latitude: Option<f32>,
    /// The maximum latitude of the bounding box, in degrees
    #[serde(rename = "maxlat", skip_serializing_if = "Option::is_none")]
    pub max_latitude: Option<f32>,
    /// The minimum longitude of the bounding box, in degrees
    #[serde(rename = "minlon", skip_serializing_if = "Option::is_none")]
    pub min_longitude: Option<f32>,
    /// The maximum longitude of the bounding box, in degrees
    #[serde(rename = "maxlon", skip_serializing_if = "Option::is_none")]
    pub max_longitude: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CircleConstraints {
    /// The latitude of the center of the circle, in degrees
    #[serde(rename = "lat", skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f32>,
    /// The longitude of the center of the circle, in degrees
    #[serde(rename = "lon", skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f32>,
    /// The minimum radius of the circle, in meters
    #[serde(rename = "minrad", skip_serializing_if = "Option::is_none")]
    pub min_radius: Option<f32>,
    /// The maximum radius of the circle, in meters
    #[serde(rename = "maxrad", skip_serializing_if = "Option::is_none")]
    pub max_radius: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OutputControl {
    /// The format of the output
    #[serde(rename = "format", default = "_get_json")]
    format: String,
    /// The HTTP status code to use for missing data
    #[serde(rename = "nodata", default = "_get_204")]
    no_data: String,
}

fn _get_json() -> String {
    "json".to_string()
}

fn _get_204() -> String {
    "204".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OtherParameters {
    /// The minimum depth to include, in kilometers
    #[serde(rename = "mindepth", skip_serializing_if = "Option::is_none")]
    pub min_depth: Option<f32>,
    /// The maximum depth to include, in kilometers
    #[serde(rename = "maxdepth", skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<f32>,
    /// The minimum magnitude to include
    #[serde(rename = "minmag", skip_serializing_if = "Option::is_none")]
    pub min_magnitude: Option<f32>,
    /// The maximum magnitude to include
    #[serde(rename = "maxmag", skip_serializing_if = "Option::is_none")]
    pub max_magnitude: Option<f32>,
    /// The type of magnitude to use, e.g. "Mw", "ML", "mb"
    #[serde(rename = "magnitudetype", skip_serializing_if = "Option::is_none")]
    pub magnitude_type: Option<String>,
    /// Whether to include all event origins
    #[serde(rename = "includeallorigns", skip_serializing_if = "Option::is_none")]
    pub include_all_orgins: Option<bool>,
    /// Whether to include arrival information
    #[serde(rename = "includearrivals", skip_serializing_if = "Option::is_none")]
    pub include_arrivals: Option<bool>,
    /// The ID of the event to include
    #[serde(rename = "eventid", skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The maximum number of results to return
    #[serde(rename = "limit", default)]
    pub limit: Limit,
    /// The number of results to skip
    #[serde(rename = "offset", skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    /// The field(s) to order the results by
    // time, time-asc, magnitude, magnitude-asc
    #[serde(rename = "orderby", skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    /// The contributor of the data
    #[serde(rename = "contributor", skip_serializing_if = "Option::is_none")]
    pub contributor: Option<String>,
    /// The data catalog to search
    #[serde(rename = "catalog", skip_serializing_if = "Option::is_none")]
    pub catalog: Option<String>,
    /// The date and time after which to include updated data
    #[serde(rename = "updatedafter", skip_serializing_if = "Option::is_none")]
    pub updated_after: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Limit(i32);

impl Default for Limit {
    fn default() -> Self {
        Limit(10)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryParams {
    #[serde(flatten)]
    pub time_constraints: TimeConstraints,
    #[serde(flatten)]
    pub box_area_constraints: BoxAreaConstraints,
    #[serde(flatten)]
    pub circle_constraints: CircleConstraints,
    #[serde(flatten, default)]
    output_control: OutputControl,
    #[serde(flatten)]
    pub other_parameters: OtherParameters,
}

impl QueryParams {
    pub fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

mod test {
    use super::{InnerWssEvent, QueryParams, WssEvent};

    const EXAMPLE_WSS: &str = r##"
    {
      "action":"create",
      "data":{
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [
          7.8865,
          46.0554,
          -8.0
        ]
      },
      "id": "20241214_0000249",
      "properties": {
        "source_id": "1744000",
        "source_catalog": "EMSC-RTS",
        "lastupdate": "2024-12-15T18:26:38.787209Z",
        "time": "2024-12-14T09:39:47.2Z",
        "flynn_region": "SWITZERLAND",
        "lat": 46.0554,
        "lon": 7.8865,
        "depth": 8.0,
        "evtype": "ke",
        "auth": "ETHZ",
        "mag": 0.9,
        "magtype": "ml",
        "unid": "20241214_0000249"
      }
    }}
    "##;

    #[test]
    fn get_empty_query() {
        let query = "{}";

        let params = serde_json::from_str::<QueryParams>(query).unwrap();
        let serialized = serde_json::to_string(&params).unwrap();

        assert_eq!(
            serialized,
            "{\"format\":\"json\",\"nodata\":\"204\",\"limit\":10}"
        )
    }

    #[test]
    fn check_wss_serde() {
        // let deserialized =
        // serde_json::from_str::<InnerWssEvent>(&EXAMPLE_WSS).unwrap();
        // println!("{deserialized:#?}");
        // let serialized = serde_json::to_string(&deserialized).unwrap();
        // println!("{serialized:#?}");
    }
}
