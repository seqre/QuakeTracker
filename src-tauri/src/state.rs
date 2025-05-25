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

    fn maybe_cleanup(&mut self) -> Result<(), PolarsError> {
        let stats = self.get_stats();
        let mut needs_cleanup = false;
        let mut cleanup_reason = String::new();

        // Check if we exceed the maximum event count
        if self.config.max_events > 0 && stats.total_events > self.config.max_events {
            needs_cleanup = true;
            cleanup_reason = format!(
                "Event count ({}) exceeds maximum ({})",
                stats.total_events, self.config.max_events
            );
        }

        // Check if we have events older than retention period
        if self.config.retention_days > 0 {
            let cutoff_time =
                chrono::Utc::now() - chrono::TimeDelta::days(self.config.retention_days as i64);
            let cutoff_ns = cutoff_time.timestamp_nanos_opt().unwrap_or(0);

            // Check if there are any old events
            let old_events_count = self
                .analytics
                .get_dataframe()
                .filter(col("time").lt(lit(cutoff_ns)))
                .select([len().alias("count")])
                .collect()?
                .column("count")?
                .u32()?
                .get(0)
                .unwrap_or(0);

            if old_events_count > 0 {
                needs_cleanup = true;
                if !cleanup_reason.is_empty() {
                    cleanup_reason.push_str(" and ");
                }
                cleanup_reason.push_str(&format!(
                    "{} events older than {} days",
                    old_events_count, self.config.retention_days
                ));
            }
        }

        if needs_cleanup {
            log::info!("Performing cleanup: {}", cleanup_reason);
            self.perform_cleanup()?;
        }

        Ok(())
    }

    /// Perform the actual cleanup by filtering the dataframe and rebuilding
    /// analytics
    fn perform_cleanup(&mut self) -> Result<(), PolarsError> {
        let old_stats = self.get_stats();
        let mut filtered_df = self.analytics.get_dataframe();

        // Apply retention period filter if configured
        if self.config.retention_days > 0 {
            let cutoff_time =
                chrono::Utc::now() - chrono::TimeDelta::days(self.config.retention_days as i64);
            let cutoff_ns = cutoff_time.timestamp_nanos_opt().unwrap_or(0);
            filtered_df = filtered_df.filter(col("time").gt_eq(lit(cutoff_ns)));
        }

        // Apply event count limit if configured
        if self.config.max_events > 0 {
            // Keep the most recent events by sorting by time descending and taking the
            // limit
            filtered_df = filtered_df
                .sort(
                    ["time"],
                    SortMultipleOptions::default().with_order_descending(true),
                )
                .limit(self.config.max_events as u32);
        }

        // Replace the dataframe and rebuild analytics
        self.analytics.replace_dataframe_and_rebuild(filtered_df)?;

        let new_stats = self.get_stats();
        log::info!(
            "Cleanup completed: {} events remaining (was {})",
            new_stats.total_events,
            old_stats.total_events
        );

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

    #[test]
    fn test_cleanup_by_event_count() {
        // Create config with low max_events for testing
        let config = DataConfig {
            max_events: 3,
            auto_cleanup: true,
            retention_days: 0, // Disable retention cleanup
        };
        let mut data = SeismicData::with_config(config);

        // Add 5 events (exceeds max_events of 3)
        let mut events = Vec::new();
        for i in 0..5 {
            let mut event = SeismicEvent::test_event();
            event.id = format!("test_{}", i);
            // Make events at different times so we can test ordering
            event.time = event.time + chrono::TimeDelta::seconds(i as i64);
            events.push(event);
        }

        data.add_events(events).unwrap();

        // Should have been cleaned up to 3 events (the most recent ones)
        let stats = data.get_stats();
        assert_eq!(stats.total_events, 3);

        // Verify the remaining events are the most recent ones
        let remaining_events = data.get_chronological_events().unwrap();
        assert_eq!(remaining_events.len(), 3);
        assert_eq!(remaining_events[0].id, "test_2");
        assert_eq!(remaining_events[1].id, "test_3");
        assert_eq!(remaining_events[2].id, "test_4");
    }

    #[test]
    fn test_cleanup_by_retention_period() {
        // Create config with short retention period for testing
        let config = DataConfig {
            max_events: 0, // Disable count-based cleanup
            auto_cleanup: true,
            retention_days: 1, // Keep only 1 day of data
        };
        let mut data = SeismicData::with_config(config);

        let now = chrono::Utc::now();
        let old_time = now - chrono::TimeDelta::days(2); // 2 days ago (should be cleaned)
        let recent_time = now - chrono::TimeDelta::hours(12); // 12 hours ago (should be kept)

        // Add old event
        let mut old_event = SeismicEvent::test_event();
        old_event.id = "old_event".to_string();
        old_event.time = old_time;

        // Add recent event
        let mut recent_event = SeismicEvent::test_event();
        recent_event.id = "recent_event".to_string();
        recent_event.time = recent_time;

        data.add_events(vec![old_event, recent_event]).unwrap();

        // Should have cleaned up the old event, keeping only the recent one
        let stats = data.get_stats();
        assert_eq!(stats.total_events, 1);

        let remaining_events = data.get_events().unwrap();
        assert_eq!(remaining_events.len(), 1);
        assert_eq!(remaining_events[0].id, "recent_event");
    }

    #[test]
    fn test_cleanup_disabled() {
        // Create config with auto_cleanup disabled
        let config = DataConfig {
            max_events: 2,
            auto_cleanup: false, // Cleanup disabled
            retention_days: 1,
        };
        let mut data = SeismicData::with_config(config);

        // Add 5 events (exceeds max_events, but cleanup is disabled)
        let mut events = Vec::new();
        for i in 0..5 {
            let mut event = SeismicEvent::test_event();
            event.id = format!("test_{}", i);
            events.push(event);
        }

        data.add_events(events).unwrap();

        // Should have all 5 events since cleanup is disabled
        let stats = data.get_stats();
        assert_eq!(stats.total_events, 5);
    }

    #[test]
    fn test_manual_cleanup() {
        // Create config with auto_cleanup disabled
        let config = DataConfig {
            max_events: 3,
            auto_cleanup: false,
            retention_days: 0,
        };
        let mut data = SeismicData::with_config(config);

        // Add 5 events
        let mut events = Vec::new();
        for i in 0..5 {
            let mut event = SeismicEvent::test_event();
            event.id = format!("test_{}", i);
            event.time = event.time + chrono::TimeDelta::seconds(i as i64);
            events.push(event);
        }

        data.add_events(events).unwrap();

        // Should have all 5 events since auto_cleanup is disabled
        let stats = data.get_stats();
        assert_eq!(stats.total_events, 5);

        // Manually trigger cleanup
        data.maybe_cleanup().unwrap();

        // Should now have only 3 events
        let stats = data.get_stats();
        assert_eq!(stats.total_events, 3);
    }

    #[test]
    fn test_config_update() {
        let mut data = SeismicData::new();

        // Add some events
        let mut events = Vec::new();
        for i in 0..5 {
            let mut event = SeismicEvent::test_event();
            event.id = format!("test_{}", i);
            events.push(event);
        }
        data.add_events(events).unwrap();

        let stats = data.get_stats();
        assert_eq!(stats.total_events, 5);

        // Update config to limit events
        let new_config = DataConfig {
            max_events: 3,
            auto_cleanup: false, // Don't auto-cleanup on config change
            retention_days: 0,
        };
        data.update_config(new_config);

        // Events should still be there since auto_cleanup is false
        let stats = data.get_stats();
        assert_eq!(stats.total_events, 5);

        // But manual cleanup should now respect the new limit
        data.maybe_cleanup().unwrap();
        let stats = data.get_stats();
        assert_eq!(stats.total_events, 3);
    }

    #[test]
    fn test_memory_usage_estimate() {
        let mut data = SeismicData::new();

        // Initially should have 0 memory usage
        let stats = data.get_stats();
        assert_eq!(stats.memory_usage_estimate, 0);

        // Add an event
        data.add_or_update_event(SeismicEvent::test_event())
            .unwrap();

        // Should have some memory usage estimate
        let stats = data.get_stats();
        assert!(stats.memory_usage_estimate > 0);
        assert_eq!(stats.memory_usage_estimate, 500); // 1 event * 500 bytes
                                                      // estimate
    }

    #[test]
    fn test_replace_dataframe_and_rebuild() {
        let analytics = crate::analytics::incremental::IncrementalAnalytics::new();

        // Add some events
        let mut events = Vec::new();
        for i in 0..5 {
            let mut event = SeismicEvent::test_event();
            event.id = format!("test_{}", i);
            event.magnitude = 2.0 + i as f64;
            events.push(event);
        }
        analytics.add_events(&events).unwrap();

        // Verify initial state
        let initial_stats = analytics.cache.read();
        assert_eq!(initial_stats.total_events, 5);
        drop(initial_stats);

        let initial_mag_dist = analytics.get_magnitude_distribution();
        assert_eq!(initial_mag_dist.len(), 5);

        // Create a filtered dataframe (keep only events with magnitude >= 4.0)
        let filtered_df = analytics.get_dataframe().filter(col("mag").gt_eq(lit(4.0)));

        // Replace dataframe and rebuild
        analytics
            .replace_dataframe_and_rebuild(filtered_df)
            .unwrap();

        // Verify the dataframe was replaced and analytics rebuilt
        let final_stats = analytics.cache.read();
        assert_eq!(final_stats.total_events, 3); // Only events with mag >= 4.0 (4.0, 5.0, 6.0)
        drop(final_stats);

        let final_mag_dist = analytics.get_magnitude_distribution();
        assert_eq!(final_mag_dist.len(), 3);

        // Verify the correct events remain
        let remaining_events = analytics.get_dataframe().collect().unwrap();
        let mags = remaining_events.column("mag").unwrap().f64().unwrap();
        for mag_opt in mags.iter() {
            if let Some(mag) = mag_opt {
                assert!(mag >= 4.0);
            }
        }
    }
}
