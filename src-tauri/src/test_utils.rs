
use chrono::{DateTime, Utc};

use crate::seismic::SeismicEvent;

pub fn create_test_event_with_params(
    id: &str,
    magnitude: f64,
    depth: f64,
    latitude: f64,
    longitude: f64,
    time: DateTime<Utc>,
    flynn_region: &str,
) -> SeismicEvent {
    let mut event = SeismicEvent::test_event();
    event.id = id.to_string();
    event.magnitude = magnitude;
    event.depth = depth;
    event.latitude = latitude;
    event.longitude = longitude;
    event.time = time;
    event.flynn_region = flynn_region.to_string();
    event
}
