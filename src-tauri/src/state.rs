use std::sync::Arc;

use polars::prelude::*;

use crate::analytics::incremental::IncrementalAnalytics;
use crate::seismic::SeismicEvent;

/// Improved seismic data storage with incremental analytics
pub struct SeismicData {
    /// Incremental analytics processor
    analytics: Arc<IncrementalAnalytics>,
    /// Configuration for data retention and processing
    config: DataConfig,
}

#[derive(Debug, Clone)]
pub struct DataConfig {
    /// Maximum number of events to keep in memory (0 = unlimited)
    pub max_events: usize,
    /// Whether to enable automatic cleanup of old events
    pub auto_cleanup: bool,
    /// Days to keep events before cleanup (if auto_cleanup is enabled)
    pub retention_days: u32,
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            max_events: 100_000, // Reasonable default for memory management
            auto_cleanup: true,
            retention_days: 365, // Keep 1 year of data by default
        }
    }
}

impl SeismicData {
    pub fn new() -> Self {
        Self::with_config(DataConfig::default())
    }

    pub fn with_config(config: DataConfig) -> Self {
        Self {
            analytics: Arc::new(IncrementalAnalytics::new()),
            config,
        }
    }

    /// Add or update a single seismic event
    pub fn add_or_update_event(&mut self, event: SeismicEvent) -> Result<(), PolarsError> {
        self.analytics.add_event(&event)?;

        // Check if cleanup is needed
        if self.config.auto_cleanup {
            self.maybe_cleanup()?;
        }

        Ok(())
    }

    /// Add multiple seismic events efficiently
    pub fn add_events(&mut self, events: Vec<SeismicEvent>) -> Result<(), PolarsError> {
        if events.is_empty() {
            return Ok(());
        }

        self.analytics.add_events(&events)?;

        // Check if cleanup is needed
        if self.config.auto_cleanup {
            self.maybe_cleanup()?;
        }

        Ok(())
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.analytics.clear();
    }

    /// Get all events (expensive operation, use sparingly)
    pub fn get_events(&self) -> Result<Vec<SeismicEvent>, PolarsError> {
        let df = self.analytics.get_dataframe().collect()?;
        self.dataframe_to_events(df)
    }

    /// Get events in chronological order (expensive operation, use sparingly)
    pub fn get_chronological_events(&self) -> Result<Vec<SeismicEvent>, PolarsError> {
        let df = self
            .analytics
            .get_dataframe()
            .sort(["time"], Default::default())
            .collect()?;
        self.dataframe_to_events(df)
    }

    /// Run a function on all events (legacy compatibility)
    pub fn run_on_events<F, T>(&self, func: F) -> Result<Vec<T>, PolarsError>
    where
        F: Fn(&SeismicEvent) -> T,
    {
        let events = self.get_events()?;
        Ok(events.iter().map(func).collect())
    }

    /// Get the underlying dataframe for advanced queries
    pub fn get_dataframe(&self) -> LazyFrame {
        self.analytics.get_dataframe()
    }

    /// Get a mutable reference to the dataframe (use with caution)
    pub fn get_mut_dataframe(&mut self) -> LazyFrame {
        self.analytics.get_dataframe()
    }

    /// Get analytics processor for direct access to incremental analytics
    pub fn get_analytics(&self) -> &IncrementalAnalytics {
        &self.analytics
    }

    /// Get current data statistics
    pub fn get_stats(&self) -> DataStats {
        let cache = self.analytics.cache.read();
        DataStats {
            total_events: cache.total_events,
            last_updated: cache.last_updated,
            memory_usage_estimate: self.estimate_memory_usage(),
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: DataConfig) {
        self.config = config;
    }

    /// Force a full recomputation of all analytics
    pub fn recompute_analytics(&self) -> Result<(), PolarsError> {
        self.analytics.recompute_all()
    }

    /// Get events within a specific time range
    pub fn get_events_in_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<SeismicEvent>, PolarsError> {
        let start_ns = start.timestamp_nanos_opt().unwrap_or(0);
        let end_ns = end.timestamp_nanos_opt().unwrap_or(0);

        let df = self
            .analytics
            .get_dataframe()
            .filter(
                col("time")
                    .gt_eq(lit(start_ns))
                    .and(col("time").lt_eq(lit(end_ns))),
            )
            .collect()?;

        self.dataframe_to_events(df)
    }

    /// Get events within a geographic bounding box
    pub fn get_events_in_bbox(
        &self,
        min_lat: f64,
        max_lat: f64,
        min_lon: f64,
        max_lon: f64,
    ) -> Result<Vec<SeismicEvent>, PolarsError> {
        let df = self
            .analytics
            .get_dataframe()
            .filter(
                col("lat")
                    .gt_eq(lit(min_lat))
                    .and(col("lat").lt_eq(lit(max_lat)))
                    .and(col("lon").gt_eq(lit(min_lon)))
                    .and(col("lon").lt_eq(lit(max_lon))),
            )
            .collect()?;

        self.dataframe_to_events(df)
    }

    /// Get events with magnitude above threshold
    pub fn get_events_above_magnitude(
        &self,
        min_magnitude: f64,
    ) -> Result<Vec<SeismicEvent>, PolarsError> {
        let df = self
            .analytics
            .get_dataframe()
            .filter(col("mag").gt_eq(lit(min_magnitude)))
            .collect()?;

        self.dataframe_to_events(df)
    }

    // Private helper methods

    fn maybe_cleanup(&self) -> Result<(), PolarsError> {
        if self.config.max_events > 0 {
            let stats = self.get_stats();
            if stats.total_events > self.config.max_events {
                // TODO: Implement cleanup logic
                // This would involve removing oldest events and updating analytics
                log::warn!(
                    "Event count ({}) exceeds maximum ({}), cleanup needed",
                    stats.total_events,
                    self.config.max_events
                );
            }
        }
        Ok(())
    }

    fn estimate_memory_usage(&self) -> usize {
        // Rough estimate: each event is approximately 500 bytes
        let cache = self.analytics.cache.read();
        cache.total_events * 500
    }

    fn dataframe_to_events(&self, df: DataFrame) -> Result<Vec<SeismicEvent>, PolarsError> {
        let mut events = Vec::new();
        let height = df.height();

        if height == 0 {
            return Ok(events);
        }

        // Extract columns
        let ids = df.column("unid")?.str()?;
        let lats = df.column("lat")?.f64()?;
        let lons = df.column("lon")?.f64()?;
        let times = df.column("time")?.datetime()?;
        let mags = df.column("mag")?.f64()?;
        let magtypes = df.column("magtype")?.str()?;
        let depths = df.column("depth")?.f64()?;
        let evtypes = df.column("evtype")?.str()?;
        let flynn_regions = df.column("flynn_region")?.str()?;
        let source_ids = df.column("source_id")?.str()?;
        let source_catalogs = df.column("source_catalog")?.str()?;
        let lastupdates = df.column("lastupdate")?.datetime()?;
        let authors = df.column("author")?.str()?;

        for i in 0..height {
            // Extract values with proper null handling
            let id = ids.get(i).map(|s| s.to_string()).unwrap_or_default();
            let latitude = lats.get(i).unwrap_or(0.0);
            let longitude = lons.get(i).unwrap_or(0.0);
            let time_ns = times.get(i).unwrap_or(0);
            let magnitude = mags.get(i).unwrap_or(0.0);
            let magnitude_type = magtypes.get(i).map(|s| s.to_string()).unwrap_or_default();
            let depth = depths.get(i).unwrap_or(0.0);
            let event_type = evtypes.get(i).map(|s| s.to_string()).unwrap_or_default();
            let flynn_region = flynn_regions
                .get(i)
                .map(|s| s.to_string())
                .unwrap_or_default();
            let source_id = source_ids.get(i).map(|s| s.to_string()).unwrap_or_default();
            let source_catalog = source_catalogs
                .get(i)
                .map(|s| s.to_string())
                .unwrap_or_default();
            let lastupdate_ns = lastupdates.get(i).unwrap_or(0);
            let author = authors.get(i).map(|s| s.to_string()).unwrap_or_default();

            // Convert timestamps back to DateTime
            let time = chrono::DateTime::from_timestamp_nanos(time_ns);
            let last_update = chrono::DateTime::from_timestamp_nanos(lastupdate_ns);

            let event = SeismicEvent {
                geometry: geo_types::Point::new(longitude, latitude),
                source_id,
                source_catalog,
                last_update,
                time,
                latitude,
                longitude,
                depth,
                event_type,
                author,
                magnitude,
                magnitude_type,
                flynn_region,
                id,
                origins: None,
                arrivals: None,
            };

            events.push(event);
        }

        Ok(events)
    }
}

impl Default for SeismicData {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the current data state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataStats {
    pub total_events: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub memory_usage_estimate: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seismic::SeismicEvent;

    #[test]
    fn test_seismic_data_creation() {
        let data = SeismicData::new();
        let stats = data.get_stats();
        assert_eq!(stats.total_events, 0);
    }

    #[test]
    fn test_add_single_event() {
        let mut data = SeismicData::new();
        let event = SeismicEvent::test_event();

        data.add_or_update_event(event).unwrap();

        let stats = data.get_stats();
        assert_eq!(stats.total_events, 1);
    }

    #[test]
    fn test_add_multiple_events() {
        let mut data = SeismicData::new();
        let events = vec![SeismicEvent::test_event(), {
            let mut event = SeismicEvent::test_event();
            event.id = "test2".to_string();
            event
        }];

        data.add_events(events).unwrap();

        let stats = data.get_stats();
        assert_eq!(stats.total_events, 2);
    }

    #[test]
    fn test_clear_data() {
        let mut data = SeismicData::new();
        data.add_or_update_event(SeismicEvent::test_event())
            .unwrap();

        data.clear();

        let stats = data.get_stats();
        assert_eq!(stats.total_events, 0);
    }
}
