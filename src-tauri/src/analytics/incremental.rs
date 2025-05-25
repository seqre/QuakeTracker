use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

use crate::analytics::processors::{
    AnalyticsProcessor, GeographicHotspotsAnalytics, GutenbergRichterAnalytics,
    MagnitudeDepthAnalytics, MagnitudeDistributionAnalytics, RiskAssessmentAnalytics,
    TemporalPatternsAnalytics,
};
use crate::seismic::SeismicEvent;

/// Generic analytics cache that stores multiple analytics processors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsCache {
    pub last_updated: DateTime<Utc>,
    pub total_events: usize,
}

impl Default for AnalyticsCache {
    fn default() -> Self {
        Self {
            last_updated: Utc::now(),
            total_events: 0,
        }
    }
}

/// Incremental analytics processor that efficiently updates computations
pub struct IncrementalAnalytics {
    /// Main dataframe containing all seismic events
    dataframe: Arc<RwLock<LazyFrame>>,
    /// Cached analytics metadata
    pub cache: Arc<RwLock<AnalyticsCache>>,
    /// Index for fast lookups by event ID
    event_index: Arc<DashMap<String, usize>>,
    /// Analytics processors
    magnitude_distribution: Arc<MagnitudeDistributionAnalytics>,
    temporal_patterns: Arc<TemporalPatternsAnalytics>,
    magnitude_depth_pairs: Arc<MagnitudeDepthAnalytics>,
    geographic_hotspots: Arc<GeographicHotspotsAnalytics>,
    gutenberg_richter: Arc<GutenbergRichterAnalytics>,
    risk_assessment: Arc<RiskAssessmentAnalytics>,
    /// List of all analytics processors for iteration
    analytics_processors: Vec<Arc<dyn AnalyticsProcessor>>,
    /// Flag to indicate if full recomputation is needed
    needs_full_recompute: Arc<AtomicBool>,
}

impl IncrementalAnalytics {
    pub fn new() -> Self {
        let magnitude_distribution = Arc::new(MagnitudeDistributionAnalytics::new());
        let temporal_patterns = Arc::new(TemporalPatternsAnalytics::new());
        let magnitude_depth_pairs = Arc::new(MagnitudeDepthAnalytics::new());
        let geographic_hotspots = Arc::new(GeographicHotspotsAnalytics::new());
        let gutenberg_richter = Arc::new(GutenbergRichterAnalytics::new());
        let risk_assessment = Arc::new(RiskAssessmentAnalytics::new());

        let analytics_processors: Vec<Arc<dyn AnalyticsProcessor>> = vec![
            magnitude_distribution.clone(),
            temporal_patterns.clone(),
            magnitude_depth_pairs.clone(),
            geographic_hotspots.clone(),
            gutenberg_richter.clone(),
            risk_assessment.clone(),
        ];

        Self {
            dataframe: Arc::new(RwLock::new(Self::empty_df())),
            cache: Arc::new(RwLock::new(AnalyticsCache::default())),
            event_index: Arc::new(DashMap::new()),
            magnitude_distribution,
            temporal_patterns,
            magnitude_depth_pairs,
            geographic_hotspots,
            gutenberg_richter,
            risk_assessment,
            analytics_processors,
            needs_full_recompute: Arc::new(AtomicBool::new(false)),
        }
    }

    fn empty_df() -> LazyFrame {
        df![
            "unid" => Vec::<String>::new(),
            "lat" => Vec::<f64>::new(),
            "lon" => Vec::<f64>::new(),
            "time" => Vec::<i64>::new(),
            "mag" => Vec::<f64>::new(),
            "magtype" => Vec::<String>::new(),
            "depth" => Vec::<f64>::new(),
            "evtype" => Vec::<String>::new(),
            "flynn_region" => Vec::<String>::new(),
            "source_id" => Vec::<String>::new(),
            "source_catalog" => Vec::<String>::new(),
            "lastupdate" => Vec::<i64>::new(),
            "author" => Vec::<String>::new(),
        ]
        .expect("Failed to create empty dataframe")
        .lazy()
        .with_columns([
            col("time").cast(DataType::Datetime(TimeUnit::Nanoseconds, None)),
            col("lastupdate").cast(DataType::Datetime(TimeUnit::Nanoseconds, None)),
        ])
    }

    /// Add a single event and update analytics incrementally
    pub fn add_event(&self, event: &SeismicEvent) -> Result<(), PolarsError> {
        let event_id = event.id.clone();

        if self.event_index.contains_key(&event_id) {
            return self.update_event(event);
        }

        let event_df = self.event_to_dataframe(event)?;

        {
            let mut df_guard = self.dataframe.write();
            *df_guard = concat([df_guard.clone(), event_df.lazy()], UnionArgs::default())?;
        }

        let new_index = self.event_index.len();
        self.event_index.insert(event_id, new_index);

        for processor in &self.analytics_processors {
            processor.update(event)?;
        }

        {
            let mut cache = self.cache.write();
            cache.last_updated = Utc::now();
            cache.total_events += 1;
        }

        Ok(())
    }

    /// Update an existing event
    pub fn update_event(&self, _event: &SeismicEvent) -> Result<(), PolarsError> {
        self.needs_full_recompute.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Add multiple events efficiently
    pub fn add_events(&self, events: &[SeismicEvent]) -> Result<(), PolarsError> {
        if events.is_empty() {
            return Ok(());
        }

        let events_df = self.events_to_dataframe(events)?;

        {
            let mut df_guard = self.dataframe.write();
            *df_guard = concat([df_guard.clone(), events_df.lazy()], UnionArgs::default())?;
        }

        let start_index = self.event_index.len();
        for (i, event) in events.iter().enumerate() {
            self.event_index.insert(event.id.clone(), start_index + i);
        }

        for event in events {
            for processor in &self.analytics_processors {
                processor.update(event)?;
            }
        }

        {
            let mut cache = self.cache.write();
            cache.last_updated = Utc::now();
            cache.total_events += events.len();
        }

        Ok(())
    }

    /// Get magnitude distribution
    pub fn get_magnitude_distribution(&self) -> Vec<(String, u32)> {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.magnitude_distribution.get_result()
    }

    /// Get count by date
    pub fn get_count_by_date(&self) -> Vec<(NaiveDate, u32)> {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.temporal_patterns.get_result()
    }

    /// Get magnitude-depth pairs
    pub fn get_mag_depth_pairs(&self) -> Vec<(f64, f64)> {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.magnitude_depth_pairs.get_result()
    }

    /// Get hourly frequency distribution
    pub fn get_hourly_frequency(&self) -> Vec<(u32, u32)> {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.temporal_patterns.get_hourly_distribution()
    }

    /// Get monthly frequency distribution
    pub fn get_monthly_frequency(&self) -> Vec<(u32, u32)> {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.temporal_patterns.get_monthly_distribution()
    }

    /// Get weekly frequency distribution with weekday names
    pub fn get_weekly_frequency(&self) -> Vec<(String, u32)> {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.temporal_patterns.get_weekly_distribution()
    }

    /// Get geographic hotspots by region
    pub fn get_region_hotspots(&self) -> Vec<(String, u32)> {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.geographic_hotspots.get_region_hotspots()
    }

    /// Get coordinate clusters for mapping
    pub fn get_coordinate_clusters(&self) -> Vec<(f64, f64, u32)> {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.geographic_hotspots.get_coordinate_clusters()
    }

    /// Get Gutenberg-Richter b-value
    pub fn get_b_value(&self) -> f64 {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.gutenberg_richter.get_b_value()
    }

    /// Get magnitude-frequency relationship data
    pub fn get_magnitude_frequency_data(&self) -> Vec<(f64, u32, u32)> {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.gutenberg_richter.get_magnitude_frequency_data()
    }

    /// Get risk assessment metrics
    pub fn get_risk_metrics(&self) -> (f64, f64, f64, f64) {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.risk_assessment.get_risk_metrics()
    }

    /// Get total seismic energy released
    pub fn get_total_energy(&self) -> f64 {
        if self.needs_full_recompute.load(Ordering::Relaxed) {
            self.recompute_all().ok();
        }
        self.risk_assessment.get_total_energy()
    }

    /// Get advanced analytics using Polars lazy evaluation
    pub fn get_advanced_analytics(&self) -> Result<AdvancedAnalytics, PolarsError> {
        let df = self.dataframe.read();
        let mut stats = Vec::new();

        // Get auxiliary stats from all processors
        for processor in &self.analytics_processors {
            let lazy_stats = processor.get_auxiliary_stats(&df);
            let collected_stats = lazy_stats.collect()?;

            // Extract title from the dataframe
            let title = if let Ok(title_col) = collected_stats.column("title") {
                if let Ok(title_str) = title_col.str() {
                    title_str.get(0).unwrap_or("Unknown").to_string()
                } else {
                    processor.name().to_string()
                }
            } else {
                processor.name().to_string()
            };

            // Convert dataframe to JSON using Polars' serde feature, excluding the title
            // column
            let data_df = collected_stats.drop("title").unwrap_or(collected_stats);
            let data = serde_json::to_value(&data_df)
                .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;

            stats.push(AnalyticsStats { title, data });
        }

        // Add regional analysis (not processor-specific)
        let regional_analysis = df
            .clone()
            .group_by([col("flynn_region")])
            .agg([
                len().alias("event_count"),
                col("mag").mean().alias("avg_magnitude"),
                col("depth").mean().alias("avg_depth"),
            ])
            .sort(
                ["event_count"],
                SortMultipleOptions::default().with_order_descending(true),
            )
            .limit(10)
            .collect()?;

        let regional_data = serde_json::to_value(&regional_analysis)
            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;
        stats.push(AnalyticsStats {
            title: "Regional Analysis".to_string(),
            data: regional_data,
        });

        Ok(AdvancedAnalytics { stats })
    }

    /// Clear all data and reset analytics
    pub fn clear(&self) {
        *self.dataframe.write() = Self::empty_df();
        *self.cache.write() = AnalyticsCache::default();
        self.event_index.clear();

        for processor in &self.analytics_processors {
            processor.clear();
        }

        self.needs_full_recompute.store(false, Ordering::Relaxed);
    }

    /// Get the underlying dataframe for custom queries
    pub fn get_dataframe(&self) -> LazyFrame {
        self.dataframe.read().clone()
    }

    /// Force a full recomputation of all analytics
    pub fn recompute_all(&self) -> Result<(), PolarsError> {
        let df = self.dataframe.read();

        for processor in &self.analytics_processors {
            processor.recompute(&df)?;
        }

        self.needs_full_recompute.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Replace the dataframe with a filtered version and rebuild analytics
    /// This is used for cleanup operations to remove old or excess events
    pub fn replace_dataframe_and_rebuild(&self, new_df: LazyFrame) -> Result<(), PolarsError> {
        // Replace the dataframe
        {
            let mut df_guard = self.dataframe.write();
            *df_guard = new_df;
        }

        // Rebuild the event index
        self.event_index.clear();
        let collected_df = self.dataframe.read().clone().collect()?;
        if let Ok(ids_column) = collected_df.column("unid") {
            if let Ok(ids) = ids_column.str() {
                for (index, id_opt) in ids.iter().enumerate() {
                    if let Some(id) = id_opt {
                        self.event_index.insert(id.to_string(), index);
                    }
                }
            }
        }

        // Update cache with new event count
        {
            let mut cache = self.cache.write();
            cache.total_events = collected_df.height();
            cache.last_updated = Utc::now();
        }

        // Clear all analytics processors and recompute
        for processor in &self.analytics_processors {
            processor.clear();
        }

        self.recompute_all()?;
        Ok(())
    }

    fn event_to_dataframe(&self, event: &SeismicEvent) -> Result<DataFrame, PolarsError> {
        let mut df = df! [
            "unid" => [event.id.as_str()],
            "lat" => [event.latitude],
            "lon" => [event.longitude],
            "time" => [event.time.timestamp_nanos_opt().unwrap_or(0)],
            "mag" => [event.magnitude],
            "magtype" => [event.magnitude_type.as_str()],
            "depth" => [event.depth],
            "evtype" => [event.event_type.as_str()],
            "flynn_region" => [event.flynn_region.as_str()],
            "source_id" => [event.source_id.as_str()],
            "source_catalog" => [event.source_catalog.as_str()],
            "lastupdate" => [event.last_update.timestamp_nanos_opt().unwrap_or(0)],
            "author" => [event.author.as_str()],
        ]?;

        df = df
            .lazy()
            .with_columns([
                col("time").cast(DataType::Datetime(TimeUnit::Nanoseconds, None)),
                col("lastupdate").cast(DataType::Datetime(TimeUnit::Nanoseconds, None)),
            ])
            .collect()?;

        Ok(df)
    }

    fn events_to_dataframe(&self, events: &[SeismicEvent]) -> Result<DataFrame, PolarsError> {
        let ids: Vec<&str> = events.iter().map(|e| e.id.as_str()).collect();
        let lats: Vec<f64> = events.iter().map(|e| e.latitude).collect();
        let lons: Vec<f64> = events.iter().map(|e| e.longitude).collect();
        let times: Vec<i64> = events
            .iter()
            .map(|e| e.time.timestamp_nanos_opt().unwrap_or(0))
            .collect();
        let mags: Vec<f64> = events.iter().map(|e| e.magnitude).collect();
        let magtypes: Vec<&str> = events.iter().map(|e| e.magnitude_type.as_str()).collect();
        let depths: Vec<f64> = events.iter().map(|e| e.depth).collect();
        let evtypes: Vec<&str> = events.iter().map(|e| e.event_type.as_str()).collect();
        let flynn_regions: Vec<&str> = events.iter().map(|e| e.flynn_region.as_str()).collect();
        let source_ids: Vec<&str> = events.iter().map(|e| e.source_id.as_str()).collect();
        let source_catalogs: Vec<&str> = events.iter().map(|e| e.source_catalog.as_str()).collect();
        let lastupdates: Vec<i64> = events
            .iter()
            .map(|e| e.last_update.timestamp_nanos_opt().unwrap_or(0))
            .collect();
        let authors: Vec<&str> = events.iter().map(|e| e.author.as_str()).collect();

        let mut df = df! [
            "unid" => ids,
            "lat" => lats,
            "lon" => lons,
            "time" => times,
            "mag" => mags,
            "magtype" => magtypes,
            "depth" => depths,
            "evtype" => evtypes,
            "flynn_region" => flynn_regions,
            "source_id" => source_ids,
            "source_catalog" => source_catalogs,
            "lastupdate" => lastupdates,
            "author" => authors,
        ]?;

        df = df
            .lazy()
            .with_columns([
                col("time").cast(DataType::Datetime(TimeUnit::Nanoseconds, None)),
                col("lastupdate").cast(DataType::Datetime(TimeUnit::Nanoseconds, None)),
            ])
            .collect()?;

        Ok(df)
    }
}

/// Advanced analytics results computed using Polars
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedAnalytics {
    pub stats: Vec<AnalyticsStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsStats {
    pub title: String,
    pub data: serde_json::Value,
}

impl AdvancedAnalytics {
    /// Convert to a serializable format for the frontend
    pub fn to_json(&self) -> Result<serde_json::Value, String> {
        serde_json::to_value(self).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;
    use crate::analytics::processors::MagnitudeDistributionAnalytics;
    use crate::seismic::SeismicEvent;
    use crate::test_utils::create_test_event_with_params;

    #[test]
    fn test_incremental_analytics_creation() {
        let analytics = IncrementalAnalytics::new();
        assert_eq!(analytics.get_magnitude_distribution().len(), 0);
        assert_eq!(analytics.get_count_by_date().len(), 0);
        assert_eq!(analytics.get_mag_depth_pairs().len(), 0);
    }

    #[test]
    fn test_analytics_processors() {
        let analytics = IncrementalAnalytics::new();

        let event = SeismicEvent::test_event();
        analytics.add_event(&event).unwrap();

        assert_eq!(analytics.cache.read().total_events, 1);
        assert!(analytics.event_index.contains_key(&event.id));

        // Test that analytics are working by checking results
        assert!(!analytics.get_magnitude_distribution().is_empty());
        assert!(!analytics.get_count_by_date().is_empty());
        assert!(!analytics.get_mag_depth_pairs().is_empty());
    }

    #[test]
    fn test_magnitude_distribution_analytics() {
        let processor = MagnitudeDistributionAnalytics::new();
        let mut event = SeismicEvent::test_event();

        event.magnitude = 2.0;
        processor.update(&event).unwrap();

        event.magnitude = 2.1;
        processor.update(&event).unwrap();

        event.magnitude = 3.0;
        processor.update(&event).unwrap();

        let distribution = processor.get_result();
        assert!(!distribution.is_empty());

        assert!(distribution
            .iter()
            .any(|(mag, count)| mag == "2" && *count == 2));
        assert!(distribution
            .iter()
            .any(|(mag, count)| mag == "3" && *count == 1));
    }

    #[test]
    fn test_analytics_cache_default() {
        let cache = AnalyticsCache::default();
        assert_eq!(cache.total_events, 0);
        // last_updated should be recent (within last second)
        let now = Utc::now();
        let diff = now.signed_duration_since(cache.last_updated);
        assert!(diff.num_seconds() < 1);
    }

    #[test]
    fn test_empty_dataframe_creation() {
        let df = IncrementalAnalytics::empty_df();
        let collected = df.collect().unwrap();

        // Should have all expected columns
        let expected_columns = vec![
            "unid",
            "lat",
            "lon",
            "time",
            "mag",
            "magtype",
            "depth",
            "evtype",
            "flynn_region",
            "source_id",
            "source_catalog",
            "lastupdate",
            "author",
        ];

        let column_names: Vec<String> = collected
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        for col_name in expected_columns {
            assert!(column_names.contains(&col_name.to_string()));
        }

        // Should be empty
        assert_eq!(collected.height(), 0);
    }

    #[test]
    fn test_add_multiple_events() {
        let analytics = IncrementalAnalytics::new();

        let events = vec![
            create_test_event_with_params("1", 2.0, 10.0, 35.0, -120.0, Utc::now(), "California"),
            create_test_event_with_params("2", 3.0, 15.0, 36.0, -121.0, Utc::now(), "California"),
            create_test_event_with_params("3", 4.0, 20.0, 37.0, -122.0, Utc::now(), "Oregon"),
        ];

        analytics.add_events(&events).unwrap();

        assert_eq!(analytics.cache.read().total_events, 3);
        assert_eq!(analytics.event_index.len(), 3);

        // Check that all events are indexed
        for event in &events {
            assert!(analytics.event_index.contains_key(&event.id));
        }

        // Check analytics results
        assert_eq!(analytics.get_magnitude_distribution().len(), 3); // 3 different magnitude buckets
        assert!(!analytics.get_count_by_date().is_empty());
        assert_eq!(analytics.get_mag_depth_pairs().len(), 3);
    }

    #[test]
    fn test_add_empty_events_list() {
        let analytics = IncrementalAnalytics::new();
        let empty_events: Vec<SeismicEvent> = vec![];

        let result = analytics.add_events(&empty_events);
        assert!(result.is_ok());
        assert_eq!(analytics.cache.read().total_events, 0);
    }

    #[test]
    fn test_update_existing_event() {
        let analytics = IncrementalAnalytics::new();

        let mut event = SeismicEvent::test_event();
        event.id = "test_event".to_string();

        // Add event first time
        analytics.add_event(&event).unwrap();
        assert_eq!(analytics.cache.read().total_events, 1);
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        // Update same event (should trigger recompute flag)
        event.magnitude = 5.0; // Change magnitude
        analytics.add_event(&event).unwrap();

        // Should still have 1 event but recompute flag should be set
        assert_eq!(analytics.cache.read().total_events, 1);
        assert!(analytics.needs_full_recompute.load(Ordering::Relaxed));
    }

    #[test]
    fn test_recompute_all() {
        let analytics = IncrementalAnalytics::new();

        // Add some events
        let events = vec![
            create_test_event_with_params("1", 2.0, 10.0, 35.0, -120.0, Utc::now(), "California"),
            create_test_event_with_params("2", 3.0, 15.0, 36.0, -121.0, Utc::now(), "Oregon"),
        ];

        analytics.add_events(&events).unwrap();

        // Set recompute flag
        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        assert!(analytics.needs_full_recompute.load(Ordering::Relaxed));

        // Recompute should clear the flag
        analytics.recompute_all().unwrap();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));
    }

    #[test]
    fn test_clear_analytics() {
        let analytics = IncrementalAnalytics::new();

        // Add some events
        let events = vec![
            create_test_event_with_params("1", 2.0, 10.0, 35.0, -120.0, Utc::now(), "California"),
            create_test_event_with_params("2", 3.0, 15.0, 36.0, -121.0, Utc::now(), "Oregon"),
        ];

        analytics.add_events(&events).unwrap();
        assert_eq!(analytics.cache.read().total_events, 2);
        assert_eq!(analytics.event_index.len(), 2);

        // Clear should reset everything
        analytics.clear();

        assert_eq!(analytics.cache.read().total_events, 0);
        assert_eq!(analytics.event_index.len(), 0);
        assert_eq!(analytics.get_magnitude_distribution().len(), 0);
        assert_eq!(analytics.get_count_by_date().len(), 0);
        assert_eq!(analytics.get_mag_depth_pairs().len(), 0);
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));
    }

    #[test]
    fn test_get_dataframe() {
        let analytics = IncrementalAnalytics::new();

        let event =
            create_test_event_with_params("1", 2.5, 12.0, 35.5, -120.5, Utc::now(), "California");
        analytics.add_event(&event).unwrap();

        let df = analytics.get_dataframe();
        let collected = df.collect().unwrap();

        assert_eq!(collected.height(), 1);

        // Check that event data is correctly stored
        let mag_col = collected.column("mag").unwrap().f64().unwrap();
        assert_eq!(mag_col.get(0), Some(2.5));

        let depth_col = collected.column("depth").unwrap().f64().unwrap();
        assert_eq!(depth_col.get(0), Some(12.0));

        let lat_col = collected.column("lat").unwrap().f64().unwrap();
        assert_eq!(lat_col.get(0), Some(35.5));

        let lon_col = collected.column("lon").unwrap().f64().unwrap();
        assert_eq!(lon_col.get(0), Some(-120.5));
    }

    #[test]
    fn test_all_analytics_methods() {
        let analytics = IncrementalAnalytics::new();

        // Create diverse events to test all analytics
        let base_time = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let events = vec![
            create_test_event_with_params("1", 2.0, 5.0, 35.0, -120.0, base_time, "California"),
            create_test_event_with_params(
                "2",
                3.0,
                15.0,
                36.0,
                -121.0,
                base_time + chrono::TimeDelta::hours(6),
                "California",
            ),
            create_test_event_with_params(
                "3",
                4.0,
                25.0,
                37.0,
                -122.0,
                base_time + chrono::TimeDelta::days(1),
                "Oregon",
            ),
            create_test_event_with_params(
                "4",
                5.0,
                35.0,
                38.0,
                -123.0,
                base_time + chrono::TimeDelta::days(32),
                "Alaska",
            ),
            create_test_event_with_params(
                "5",
                6.0,
                45.0,
                39.0,
                -124.0,
                base_time + chrono::TimeDelta::days(3),
                "Washington",
            ),
        ];

        analytics.add_events(&events).unwrap();

        // Test magnitude distribution
        let mag_dist = analytics.get_magnitude_distribution();
        assert!(!mag_dist.is_empty());
        assert_eq!(mag_dist.len(), 5); // 5 different magnitude buckets

        // Test temporal analytics
        let count_by_date = analytics.get_count_by_date();
        assert!(!count_by_date.is_empty());
        assert!(count_by_date.len() >= 3); // At least 3 different dates

        let hourly_freq = analytics.get_hourly_frequency();
        assert!(!hourly_freq.is_empty());

        let monthly_freq = analytics.get_monthly_frequency();
        assert!(!monthly_freq.is_empty());
        assert!(monthly_freq.len() >= 2); // January and February

        let weekly_freq = analytics.get_weekly_frequency();
        assert!(!weekly_freq.is_empty());

        // Test magnitude-depth pairs
        let mag_depth_pairs = analytics.get_mag_depth_pairs();
        assert_eq!(mag_depth_pairs.len(), 5);

        // Test geographic analytics
        let region_hotspots = analytics.get_region_hotspots();
        assert!(!region_hotspots.is_empty());
        assert!(region_hotspots.len() >= 4); // At least 4 different regions

        let coordinate_clusters = analytics.get_coordinate_clusters();
        assert!(!coordinate_clusters.is_empty());

        // Test Gutenberg-Richter analytics
        let b_value = analytics.get_b_value();
        assert!(b_value > 0.0);

        let mag_freq_data = analytics.get_magnitude_frequency_data();
        assert!(!mag_freq_data.is_empty());

        // Test risk assessment
        let (prob_5_30, prob_6_365, prob_7_365, total_energy) = analytics.get_risk_metrics();
        assert!(prob_5_30 >= 0.0 && prob_5_30 <= 1.0);
        assert!(prob_6_365 >= 0.0 && prob_6_365 <= 1.0);
        assert!(prob_7_365 >= 0.0 && prob_7_365 <= 1.0);
        assert!(total_energy > 0.0);

        let energy = analytics.get_total_energy();
        assert_eq!(energy, total_energy);
    }

    #[test]
    fn test_advanced_analytics_structure() {
        let analytics = IncrementalAnalytics::new();

        // Add some events
        let events = vec![
            create_test_event_with_params("1", 2.0, 10.0, 35.0, -120.0, Utc::now(), "California"),
            create_test_event_with_params("2", 3.0, 15.0, 36.0, -121.0, Utc::now(), "Oregon"),
            create_test_event_with_params("3", 4.0, 20.0, 37.0, -122.0, Utc::now(), "Washington"),
        ];

        analytics.add_events(&events).unwrap();

        let advanced_analytics = analytics.get_advanced_analytics().unwrap();

        // Should have stats from all processors plus regional analysis
        assert!(advanced_analytics.stats.len() >= 6); // 6 processors + regional analysis

        // Check that each stat has required fields
        for stat in &advanced_analytics.stats {
            assert!(!stat.title.is_empty());
            assert!(stat.data.is_object() || stat.data.is_array());
        }

        // Check for expected analytics titles
        let titles: Vec<&str> = advanced_analytics
            .stats
            .iter()
            .map(|s| s.title.as_str())
            .collect();
        assert!(titles.contains(&"Magnitude Statistics"));
        assert!(titles.contains(&"Temporal Patterns Analysis"));
        assert!(titles.contains(&"Depth Statistics"));
        assert!(titles.contains(&"Geographic Hotspots"));
        assert!(titles.contains(&"Gutenberg-Richter Analysis"));
        assert!(titles.contains(&"Risk Assessment"));
        assert!(titles.contains(&"Regional Analysis"));
    }

    #[test]
    fn test_advanced_analytics_to_json() {
        let analytics = IncrementalAnalytics::new();

        let event =
            create_test_event_with_params("1", 2.5, 12.0, 35.0, -120.0, Utc::now(), "California");
        analytics.add_event(&event).unwrap();

        let advanced_analytics = analytics.get_advanced_analytics().unwrap();
        let json_result = advanced_analytics.to_json();

        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();

        // Should be a JSON object with stats array
        assert!(json_value.is_object());
        let obj = json_value.as_object().unwrap();
        assert!(obj.contains_key("stats"));

        let stats_array = obj.get("stats").unwrap().as_array().unwrap();
        assert!(!stats_array.is_empty());
    }

    #[test]
    fn test_event_to_dataframe_conversion() {
        let analytics = IncrementalAnalytics::new();

        let event = create_test_event_with_params(
            "test_123",
            4.5,
            25.5,
            40.123,
            -125.456,
            DateTime::parse_from_rfc3339("2024-06-15T14:30:45Z")
                .unwrap()
                .with_timezone(&Utc),
            "Test Region",
        );

        let df = analytics.event_to_dataframe(&event).unwrap();

        assert_eq!(df.height(), 1);

        // Check all fields are correctly converted
        assert_eq!(
            df.column("unid").unwrap().str().unwrap().get(0),
            Some("test_123")
        );
        assert_eq!(df.column("mag").unwrap().f64().unwrap().get(0), Some(4.5));
        assert_eq!(
            df.column("depth").unwrap().f64().unwrap().get(0),
            Some(25.5)
        );
        assert_eq!(
            df.column("lat").unwrap().f64().unwrap().get(0),
            Some(40.123)
        );
        assert_eq!(
            df.column("lon").unwrap().f64().unwrap().get(0),
            Some(-125.456)
        );
        assert_eq!(
            df.column("flynn_region").unwrap().str().unwrap().get(0),
            Some("Test Region")
        );
    }

    #[test]
    fn test_events_to_dataframe_conversion() {
        let analytics = IncrementalAnalytics::new();

        let events = vec![
            create_test_event_with_params("1", 2.0, 10.0, 35.0, -120.0, Utc::now(), "California"),
            create_test_event_with_params("2", 3.0, 15.0, 36.0, -121.0, Utc::now(), "Oregon"),
            create_test_event_with_params("3", 4.0, 20.0, 37.0, -122.0, Utc::now(), "Washington"),
        ];

        let df = analytics.events_to_dataframe(&events).unwrap();

        assert_eq!(df.height(), 3);

        // Check that all events are included
        let ids = df.column("unid").unwrap().str().unwrap();
        assert_eq!(ids.get(0), Some("1"));
        assert_eq!(ids.get(1), Some("2"));
        assert_eq!(ids.get(2), Some("3"));

        let mags = df.column("mag").unwrap().f64().unwrap();
        assert_eq!(mags.get(0), Some(2.0));
        assert_eq!(mags.get(1), Some(3.0));
        assert_eq!(mags.get(2), Some(4.0));
    }

    #[test]
    fn test_recompute_triggers() {
        let analytics = IncrementalAnalytics::new();

        // Add events
        let events = vec![
            create_test_event_with_params("1", 2.0, 10.0, 35.0, -120.0, Utc::now(), "California"),
            create_test_event_with_params("2", 3.0, 15.0, 36.0, -121.0, Utc::now(), "Oregon"),
        ];

        analytics.add_events(&events).unwrap();

        // Set recompute flag
        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);

        // Any get method should trigger recompute and clear the flag
        let _ = analytics.get_magnitude_distribution();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        // Set flag again and test with different method
        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        let _ = analytics.get_count_by_date();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        // Test with all other methods
        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        let _ = analytics.get_mag_depth_pairs();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        let _ = analytics.get_hourly_frequency();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        let _ = analytics.get_monthly_frequency();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        let _ = analytics.get_weekly_frequency();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        let _ = analytics.get_region_hotspots();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        let _ = analytics.get_coordinate_clusters();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        let _ = analytics.get_b_value();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        let _ = analytics.get_magnitude_frequency_data();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        let _ = analytics.get_risk_metrics();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));

        analytics
            .needs_full_recompute
            .store(true, Ordering::Relaxed);
        let _ = analytics.get_total_energy();
        assert!(!analytics.needs_full_recompute.load(Ordering::Relaxed));
    }

    #[test]
    fn test_analytics_stats_serialization() {
        let stats = AnalyticsStats {
            title: "Test Analytics".to_string(),
            data: serde_json::json!({
                "mean": 3.5,
                "count": 10,
                "values": [1, 2, 3, 4, 5]
            }),
        };

        // Test serialization
        let serialized = serde_json::to_string(&stats).unwrap();
        assert!(serialized.contains("Test Analytics"));
        assert!(serialized.contains("mean"));
        assert!(serialized.contains("3.5"));

        // Test deserialization
        let deserialized: AnalyticsStats = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.title, "Test Analytics");
        assert_eq!(deserialized.data["mean"], 3.5);
        assert_eq!(deserialized.data["count"], 10);
    }

    #[test]
    fn test_advanced_analytics_serialization() {
        let advanced_analytics = AdvancedAnalytics {
            stats: vec![
                AnalyticsStats {
                    title: "Test 1".to_string(),
                    data: serde_json::json!({"value": 1}),
                },
                AnalyticsStats {
                    title: "Test 2".to_string(),
                    data: serde_json::json!({"value": 2}),
                },
            ],
        };

        // Test serialization
        let serialized = serde_json::to_string(&advanced_analytics).unwrap();
        assert!(serialized.contains("Test 1"));
        assert!(serialized.contains("Test 2"));

        // Test deserialization
        let deserialized: AdvancedAnalytics = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.stats.len(), 2);
        assert_eq!(deserialized.stats[0].title, "Test 1");
        assert_eq!(deserialized.stats[1].title, "Test 2");

        // Test to_json method
        let json_value = advanced_analytics.to_json().unwrap();
        assert!(json_value.is_object());
        let obj = json_value.as_object().unwrap();
        assert!(obj.contains_key("stats"));
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let analytics = Arc::new(IncrementalAnalytics::new());
        let mut handles = vec![];

        // Spawn multiple threads to add events concurrently
        for i in 0..5 {
            let analytics_clone = analytics.clone();
            let handle = thread::spawn(move || {
                let event = create_test_event_with_params(
                    &format!("thread_{}", i),
                    2.0 + i as f64,
                    10.0 + i as f64,
                    35.0 + i as f64,
                    -120.0 - i as f64,
                    Utc::now(),
                    "California",
                );
                analytics_clone.add_event(&event).unwrap();
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Check that all events were added
        assert_eq!(analytics.cache.read().total_events, 5);
        assert_eq!(analytics.event_index.len(), 5);

        // Check that analytics work correctly
        let mag_dist = analytics.get_magnitude_distribution();
        assert_eq!(mag_dist.len(), 5); // 5 different magnitudes
    }

    #[test]
    fn test_large_dataset_performance() {
        let analytics = IncrementalAnalytics::new();

        // Create a larger dataset
        let mut events = Vec::new();
        let base_time = Utc::now();

        for i in 0..100 {
            let event = create_test_event_with_params(
                &format!("event_{}", i),
                2.0 + (i % 50) as f64 / 10.0, // Magnitudes from 2.0 to 6.9
                5.0 + (i % 40) as f64,        // Depths from 5.0 to 44.0
                35.0 + (i % 10) as f64 / 10.0, // Latitudes around 35.0
                -120.0 - (i % 10) as f64 / 10.0, // Longitudes around -120.0
                base_time + chrono::TimeDelta::hours(i as i64),
                if i % 3 == 0 {
                    "California"
                } else if i % 3 == 1 {
                    "Oregon"
                } else {
                    "Washington"
                },
            );
            events.push(event);
        }

        // Add all events at once
        let start = std::time::Instant::now();
        analytics.add_events(&events).unwrap();
        let duration = start.elapsed();

        // Should complete reasonably quickly (less than 1 second for 100 events)
        assert!(duration.as_secs() < 1);

        // Verify all analytics work with larger dataset
        assert_eq!(analytics.cache.read().total_events, 100);
        assert!(!analytics.get_magnitude_distribution().is_empty());
        assert!(!analytics.get_count_by_date().is_empty());
        assert_eq!(analytics.get_mag_depth_pairs().len(), 100);

        // Test advanced analytics with larger dataset
        let advanced = analytics.get_advanced_analytics().unwrap();
        assert!(!advanced.stats.is_empty());
    }
}
