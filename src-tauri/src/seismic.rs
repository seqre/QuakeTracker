use std::io::Cursor;

use chrono::{DateTime, Utc};
use geojson::de::deserialize_geometry;
use geojson::ser::serialize_geometry;
use serde::{Deserialize, Serialize};

// Generated from: https://www.emsc-csem.org/Files/epos/specifications/Specs_fdsnevent-WS.pdf

/// Main event feature representing an earthquake event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeismicEvent {
    #[serde(
        serialize_with = "serialize_geometry",
        deserialize_with = "deserialize_geometry"
    )]
    pub geometry: geo_types::Point<f64>,
    #[serde(rename = "source_id")]
    pub source_id: String,
    #[serde(rename = "source_catalog")]
    pub source_catalog: String,
    #[serde(rename = "lastupdate")]
    pub last_update: DateTime<Utc>,
    #[serde(rename = "time")]
    pub time: DateTime<Utc>,
    #[serde(rename = "lat")]
    pub latitude: f64,
    #[serde(rename = "lon")]
    pub longitude: f64,
    #[serde(rename = "depth")]
    pub depth: f64,
    #[serde(rename = "evtype")]
    pub event_type: String,
    #[serde(rename = "auth")]
    pub author: String,
    #[serde(rename = "mag")]
    pub magnitude: f64,
    #[serde(rename = "magtype")]
    pub magnitude_type: String,
    #[serde(rename = "flynn_region")]
    pub flynn_region: String,
    #[serde(rename = "unid")]
    pub id: String,
    pub origins: Option<OriginCollection>,
    #[serde(default)]
    pub arrivals: Option<Vec<Arrival>>,
}

impl SeismicEvent {
    pub(crate) fn test_event() -> Self {
        let js = r##"
        {
          "type": "Feature",
          "geometry": {
            "type": "Point",
            "coordinates": [
              -155.4875,
              18.8232,
              -16.1
            ]
          },
          "id": "20241210_0000315",
          "properties": {
            "source_id": "1741830",
            "source_catalog": "EMSC-RTS",
            "lastupdate": "2024-12-10T22:30:25.164009Z",
            "time": "2024-12-10T22:28:31.49Z",
            "flynn_region": "HAWAII REGION, HAWAII",
            "lat": 18.8232,
            "lon": -155.4875,
            "depth": 16.1,
            "evtype": "ke",
            "auth": "HV",
            "mag": 2,
            "magtype": "md",
            "unid": "20241210_0000315"
          }
        }
        "##;

        let cursor = Cursor::new(js);

        geojson::de::deserialize_single_feature(cursor).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginCollection {
    #[serde(
        serialize_with = "serialize_geometry",
        deserialize_with = "deserialize_geometry"
    )]
    geometry: geo_types::Point<f64>,
    #[serde(default)]
    pub origins: Vec<Origin>,
}

/// Origin object representing details of the earthquake origin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Origin {
    #[serde(rename = "Source_id")]
    pub source_id: String,
    #[serde(rename = "Source_catalog")]
    pub source_catalog: String,
    #[serde(rename = "Lastupdate")]
    pub last_update: DateTime<Utc>,
    #[serde(rename = "Time")]
    pub time: DateTime<Utc>,
    #[serde(rename = "Lat")]
    pub latitude: f64,
    #[serde(rename = "Lon")]
    pub longitude: f64,
    #[serde(rename = "Depth")]
    pub depth: f64,
    #[serde(rename = "Evtype")]
    pub event_type: Option<String>,
    #[serde(rename = "Auth")]
    pub author: Option<String>,
    #[serde(rename = "Ndef")]
    pub number_of_phases: Option<i32>,
    #[serde(rename = "Nsta")]
    pub number_of_stations: Option<i32>,
    #[serde(rename = "Gap")]
    pub azimuthal_gap: Option<f64>,
    #[serde(rename = "Rms")]
    pub standard_error: Option<f64>,
    #[serde(rename = "Stime")]
    pub time_uncertainty: Option<f64>,
    #[serde(rename = "Smajor")]
    pub semi_major_axis: Option<f64>,
    #[serde(rename = "Sminor")]
    pub semi_minor_axis: Option<f64>,
    #[serde(rename = "azimut")]
    pub major_axis_azimuth: Option<f64>,
    #[serde(rename = "Sdepth")]
    pub depth_uncertainty: Option<f64>,
    #[serde(rename = "Mindist")]
    pub minimum_distance: Option<f64>,
    #[serde(rename = "Maxdist")]
    pub maximum_distance: Option<f64>,
    #[serde(rename = "Antype")]
    pub evaluation_mode: Option<String>,
    #[serde(rename = "Loctype")]
    pub location_method: Option<String>,
    #[serde(default)]
    pub mags: Vec<Magnitude>,
}

/// Magnitude object representing earthquake magnitude details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Magnitude {
    #[serde(rename = "Value")]
    pub value: f64,
    #[serde(rename = "Type")]
    pub magnitude_type: String,
    #[serde(rename = "Nsta")]
    pub station_count: Option<i32>,
    #[serde(rename = "Error")]
    pub uncertainty: Option<f64>,
    #[serde(rename = "Rang")]
    pub rang: Option<i32>,
}

/// Arrival object representing seismic wave arrival details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arrival {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "Sta")]
    pub station: String,
    #[serde(rename = "Dist")]
    pub distance: Option<f64>,
    #[serde(rename = "Evaz")]
    pub event_azimuth: Option<f64>,
    #[serde(rename = "Picktype")]
    pub pick_type: Option<String>,
    #[serde(rename = "Direction")]
    pub pick_direction: Option<String>,
    #[serde(rename = "Detchar")]
    pub pick_onset: Option<String>,
    #[serde(rename = "Phase")]
    pub phase_name: Option<String>,
    #[serde(rename = "Datetime")]
    pub datetime: Option<String>,
    #[serde(rename = "Timeres")]
    pub time_residual: Option<f64>,
    #[serde(rename = "Azim")]
    pub back_azimuth: Option<f64>,
    #[serde(rename = "Azres")]
    pub back_azimuth_residual: Option<f64>,
    #[serde(rename = "Slow")]
    pub horizontal_slowness: Option<f64>,
    #[serde(rename = "Sres")]
    pub horizontal_slowness_residual: Option<f64>,
    #[serde(rename = "Tdef")]
    pub time_used: Option<String>,
    #[serde(rename = "Adef")]
    pub back_azimuth_used: Option<String>,
    #[serde(rename = "Sdef")]
    pub slowness_used: Option<String>,
    #[serde(rename = "Snr")]
    pub signal_to_noise_ratio: Option<f64>,
    #[serde(rename = "Amp")]
    pub amplitude: Option<f64>,
    #[serde(rename = "Per")]
    pub period: Option<f64>,
    #[serde(default)]
    pub stamag: Vec<StamagObject>,
}

/// Stamag object (not detailed in the specification, added as a placeholder)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StamagObject {
    // Placeholder for potential fields
}

mod test {
    use chrono::{DateTime, NaiveDate, Utc};

    use crate::seismic::SeismicEvent;

    const EXAMPLE_JSON: &'static str = r##"
    {
      "type": "FeatureCollection",
      "metadata": {
        "count": 2
      },
      "features": [
        {
          "type": "Feature",
          "geometry": {
            "type": "Point",
            "coordinates": [
              -155.4875,
              18.8232,
              -16.1
            ]
          },
          "id": "20241210_0000315",
          "properties": {
            "source_id": "1741830",
            "source_catalog": "EMSC-RTS",
            "lastupdate": "2024-12-10T22:30:25.164009Z",
            "time": "2024-12-10T22:28:31.49Z",
            "flynn_region": "HAWAII REGION, HAWAII",
            "lat": 18.8232,
            "lon": -155.4875,
            "depth": 16.1,
            "evtype": "ke",
            "auth": "HV",
            "mag": 2,
            "magtype": "md",
            "unid": "20241210_0000315"
          }
        },
        {
          "type": "Feature",
          "geometry": {
            "type": "Point",
            "coordinates": [
              22.36,
              38.49,
              -5
            ]
          },
          "id": "20241210_0000314",
          "properties": {
            "source_id": "1741829",
            "source_catalog": "EMSC-RTS",
            "lastupdate": "2024-12-10T22:28:22.145984Z",
            "time": "2024-12-10T22:25:50.4Z",
            "flynn_region": "GREECE",
            "lat": 38.49,
            "lon": 22.36,
            "depth": 5,
            "evtype": "ke",
            "auth": "THE",
            "mag": 2.1,
            "magtype": "ml",
            "unid": "20241210_0000314"
          }
        }
      ]
    }
    }
    "##;

    // "2024-12-10T22:28:31.49Z"
    const FIRST_DATE: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
        NaiveDate::from_ymd_opt(2024, 12, 10)
            .unwrap()
            .and_hms_milli_opt(22, 28, 31, 490)
            .unwrap(),
        Utc,
    );

    #[test]
    fn check_deserialize() {
        let feature_collection: Vec<SeismicEvent> =
            geojson::de::deserialize_feature_collection_str_to_vec(&EXAMPLE_JSON).unwrap();
        assert_eq!(feature_collection.len(), 2);
        assert_eq!(feature_collection[0].id, String::from("20241210_0000315"));
        assert_eq!(feature_collection[0].time, FIRST_DATE);
        assert!(feature_collection[1].origins.is_none());
    }
}
