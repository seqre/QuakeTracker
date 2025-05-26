use chrono::{DateTime, Utc};
use geojson::JsonValue;
use serde::{Deserialize, Serialize};

use crate::error::{ErrorContextExt, QuakeTrackerError, Result};
use crate::seismic::SeismicEvent;
use crate::AppState;

pub(crate) static SEISMIC_URL: &str = "https://www.seismicportal.eu/fdsnws/event/1/query";
pub(crate) static SEISMIC_WSS_URL: &str = "wss://www.seismicportal.eu/standing_order/websocket";

/// Client error type for Tauri command responses
/// 
/// This error type is specifically designed for serialization to the frontend
/// and provides a clean interface for error handling in Tauri commands.
/// It uses tagged serialization to provide structured error information.
#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "type", content = "message")]
pub enum ClientError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("IPC error: {0}")]
    Ipc(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<QuakeTrackerError> for ClientError {
    fn from(err: QuakeTrackerError) -> Self {
        match err {
            QuakeTrackerError::Validation { message, .. } => ClientError::Validation(message),
            QuakeTrackerError::Network(_) => ClientError::Network(err.to_string()),
            QuakeTrackerError::Json(_) | QuakeTrackerError::GeoJson(_) | QuakeTrackerError::DateTime(_) => {
                ClientError::Parse(err.to_string())
            }
            QuakeTrackerError::ExternalService { message, .. } => ClientError::Network(message),
            QuakeTrackerError::Analytics(_) => ClientError::Internal(err.to_string()),
            QuakeTrackerError::Storage(_) => ClientError::Internal(err.to_string()),
            QuakeTrackerError::State(_) => ClientError::Internal(err.to_string()),
            QuakeTrackerError::Configuration { message } => ClientError::Internal(message),
            QuakeTrackerError::ResourceExhaustion { message, .. } => ClientError::Internal(message),
            QuakeTrackerError::Internal { message } => ClientError::Internal(message),
        }
    }
}

/// Result type alias for client operations
pub type ClientResult<T> = std::result::Result<T, ClientError>;

pub(crate) async fn get_seismic_events_internal(
    state: &AppState,
    query_params: QueryParams,
) -> ClientResult<String> {
    let result = get_seismic_events_internal_impl(state, query_params).await;
    result.map_err(|e| e.into())
}

async fn get_seismic_events_internal_impl(
    state: &AppState,
    query_params: QueryParams,
) -> Result<String> {
    query_params.validate()
        .with_operation("validate_params", "client")?;

    let response = reqwest::Client::new()
        .get(SEISMIC_URL)
        .query(&query_params)
        .send()
        .await
        .with_operation("fetch_events", "emsc_api")?;

    let events = response
        .text()
        .await
        .with_operation("read_response", "emsc_api")?;

    let parsed: Vec<SeismicEvent> = geojson::de::deserialize_feature_collection_str_to_vec(&events)
        .with_operation("parse_geojson", "client")?;

    let mut state = state
        .lock()
        .map_err(|e| QuakeTrackerError::state(format!("Failed to acquire state lock: {}", e)))?;
    
    state
        .add_events(parsed)
        .with_operation("store_events", "state")?;

    Ok(events)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WssAction {
    Create,
    Update,
}

#[derive(Debug, Clone, Deserialize)]
struct InnerWssEvent {
    pub action: WssAction,
    pub data: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "InnerWssEvent", rename_all(serialize = "camelCase"))]
pub struct WssEvent {
    pub action: WssAction,
    pub data: SeismicEvent,
}

impl From<InnerWssEvent> for WssEvent {
    fn from(inner: InnerWssEvent) -> Self {
        let reader = inner.data.to_string();
        let event = geojson::de::deserialize_single_feature(reader.as_bytes()).unwrap();
        WssEvent {
            action: inner.action,
            data: event,
        }
    }
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
    pub fn validate(&self) -> Result<()> {
        use crate::error::validation::*;

        // Validate time constraints
        if let (Some(start), Some(end)) = (&self.time_constraints.start_time, &self.time_constraints.end_time) {
            if start > end {
                return Err(QuakeTrackerError::validation(
                    "time_range",
                    "Start time must be before end time"
                ));
            }
        }

        // Validate geographic constraints (bounding box)
        if let Some(min_lat) = self.box_area_constraints.min_latitude {
            validate_latitude(min_lat as f64)?;
        }
        if let Some(max_lat) = self.box_area_constraints.max_latitude {
            validate_latitude(max_lat as f64)?;
        }
        if let Some(min_lon) = self.box_area_constraints.min_longitude {
            validate_longitude(min_lon as f64)?;
        }
        if let Some(max_lon) = self.box_area_constraints.max_longitude {
            validate_longitude(max_lon as f64)?;
        }

        // Validate bounding box consistency
        if let (Some(min_lat), Some(max_lat)) = (
            self.box_area_constraints.min_latitude,
            self.box_area_constraints.max_latitude,
        ) {
            if min_lat > max_lat {
                return Err(QuakeTrackerError::validation(
                    "latitude_range",
                    "Minimum latitude must be less than maximum latitude"
                ));
            }
        }

        if let (Some(min_lon), Some(max_lon)) = (
            self.box_area_constraints.min_longitude,
            self.box_area_constraints.max_longitude,
        ) {
            if min_lon > max_lon {
                return Err(QuakeTrackerError::validation(
                    "longitude_range",
                    "Minimum longitude must be less than maximum longitude"
                ));
            }
        }

        // Validate circular constraints
        if let Some(lat) = self.circle_constraints.latitude {
            validate_latitude(lat as f64)?;
        }
        if let Some(lon) = self.circle_constraints.longitude {
            validate_longitude(lon as f64)?;
        }

        if let (Some(min_rad), Some(max_rad)) = (
            self.circle_constraints.min_radius,
            self.circle_constraints.max_radius,
        ) {
            if min_rad < 0.0 {
                return Err(QuakeTrackerError::validation(
                    "min_radius",
                    "Minimum radius cannot be negative"
                ));
            }
            if max_rad < 0.0 {
                return Err(QuakeTrackerError::validation(
                    "max_radius",
                    "Maximum radius cannot be negative"
                ));
            }
            if min_rad > max_rad {
                return Err(QuakeTrackerError::validation(
                    "radius_range",
                    "Minimum radius must be less than maximum radius"
                ));
            }
        }

        // Validate depth constraints
        if let Some(min_depth) = self.other_parameters.min_depth {
            validate_depth(min_depth as f64)?;
        }
        if let Some(max_depth) = self.other_parameters.max_depth {
            validate_depth(max_depth as f64)?;
        }

        if let (Some(min_depth), Some(max_depth)) = (
            self.other_parameters.min_depth,
            self.other_parameters.max_depth,
        ) {
            if min_depth > max_depth {
                return Err(QuakeTrackerError::validation(
                    "depth_range",
                    "Minimum depth must be less than maximum depth"
                ));
            }
        }

        // Validate magnitude constraints
        if let Some(min_mag) = self.other_parameters.min_magnitude {
            validate_magnitude(min_mag as f64)?;
        }
        if let Some(max_mag) = self.other_parameters.max_magnitude {
            validate_magnitude(max_mag as f64)?;
        }

        if let (Some(min_mag), Some(max_mag)) = (
            self.other_parameters.min_magnitude,
            self.other_parameters.max_magnitude,
        ) {
            if min_mag > max_mag {
                return Err(QuakeTrackerError::validation(
                    "magnitude_range",
                    "Minimum magnitude must be less than maximum magnitude"
                ));
            }
        }

        // Validate limit
        if self.other_parameters.limit.0 <= 0 {
            return Err(QuakeTrackerError::validation(
                "limit",
                "Limit must be greater than 0"
            ));
        }

        if self.other_parameters.limit.0 > 20000 {
            return Err(QuakeTrackerError::validation(
                "limit",
                "Limit cannot exceed 20000 events"
            ));
        }

        // Validate offset
        if let Some(offset) = self.other_parameters.offset {
            if offset < 0 {
                return Err(QuakeTrackerError::validation(
                    "offset",
                    "Offset cannot be negative"
                ));
            }
        }

        // Validate event ID
        if let Some(ref event_id) = self.other_parameters.event_id {
            validate_event_id(event_id)?;
        }

        Ok(())
    }
}

mod test {
    use crate::client::{QueryParams, WssAction, WssEvent};

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
        let deserialized = serde_json::from_str::<WssEvent>(&EXAMPLE_WSS).unwrap();
        assert_eq!(deserialized.action, WssAction::Create);
    }
}
