use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate, Timelike, Weekday};
use itertools::Itertools;
use parking_lot::RwLock;
use polars::prelude::*;

use crate::seismic::SeismicEvent;

/// Trait for analytics that can be incrementally updated
pub trait AnalyticsProcessor: Send + Sync {
    /// Get the name/identifier for this analytics processor
    fn name(&self) -> &'static str;

    /// Update analytics with a new event
    fn update(&self, event: &SeismicEvent) -> Result<(), PolarsError>;

    /// Recompute analytics from the dataframe
    fn recompute(&self, dataframe: &LazyFrame) -> Result<(), PolarsError>;

    /// Clear all cached data
    fn clear(&self);

    /// Get auxiliary statistics as a LazyFrame for advanced analytics
    fn get_auxiliary_stats(&self, dataframe: &LazyFrame) -> LazyFrame;
}

/// Magnitude distribution analytics processor
///
/// This processor analyzes the distribution of earthquake magnitudes by
/// grouping them into buckets (bins) to create a histogram. It uses 0.2
/// magnitude unit buckets (e.g., 2.0-2.2, 2.2-2.4, etc.) to provide a detailed
/// view of magnitude frequency.
///
/// The analysis helps identify:
/// - Most common magnitude ranges
/// - Distribution shape (exponential, normal, etc.)
pub struct MagnitudeDistributionAnalytics {
    buckets: Arc<RwLock<HashMap<u32, u32>>>,
}

impl MagnitudeDistributionAnalytics {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_result(&self) -> Result<Vec<(String, u32)>, String> {
        let buckets = self.buckets.read();
        let mut result: Vec<_> = buckets
            .iter()
            .map(|(bucket, count)| (((*bucket as f32) / 10.0).to_string(), *count))
            .collect();

        result.sort_by(|a, b| {
            let a_val =
                a.0.parse::<f32>()
                    .map_err(|e| format!("Failed to parse magnitude '{}': {}", a.0, e));
            let b_val =
                b.0.parse::<f32>()
                    .map_err(|e| format!("Failed to parse magnitude '{}': {}", b.0, e));

            match (a_val, b_val) {
                (Ok(a), Ok(b)) => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
                _ => std::cmp::Ordering::Equal, // This shouldn't happen with our data
            }
        });

        Ok(result)
    }
}

impl AnalyticsProcessor for MagnitudeDistributionAnalytics {
    fn name(&self) -> &'static str {
        "magnitude_distribution"
    }

    fn update(&self, event: &SeismicEvent) -> Result<(), PolarsError> {
        let bucket = ((event.magnitude * 10.0) as u32) - (((event.magnitude * 10.0) as u32) % 2);
        let mut buckets = self.buckets.write();
        *buckets.entry(bucket).or_insert(0) += 1;
        Ok(())
    }

    fn recompute(&self, dataframe: &LazyFrame) -> Result<(), PolarsError> {
        let result = dataframe.clone().select([col("mag")]).collect()?;

        let magnitudes = result.column("mag")?.f64()?;
        let mut buckets = HashMap::new();

        for mag_opt in magnitudes.iter() {
            if let Some(mag) = mag_opt {
                let bucket = ((mag * 10.0) as u32) - (((mag * 10.0) as u32) % 2);
                *buckets.entry(bucket).or_insert(0) += 1;
            }
        }

        *self.buckets.write() = buckets;
        Ok(())
    }

    fn clear(&self) {
        self.buckets.write().clear();
    }

    fn get_auxiliary_stats(&self, dataframe: &LazyFrame) -> LazyFrame {
        dataframe
            .clone()
            .select([
                col("mag").mean().alias("mean_magnitude"),
                col("mag").median().alias("median_magnitude"),
                col("mag").std(1).alias("std_magnitude"),
                col("mag").min().alias("min_magnitude"),
                col("mag").max().alias("max_magnitude"),
            ])
            .with_columns([lit("Magnitude Statistics").alias("title")])
    }
}

/// Unified temporal patterns analytics processor
///
/// This processor provides comprehensive temporal analysis of earthquake
/// occurrence patterns across multiple time scales. It combines daily tracking
/// with frequency distribution analysis to identify various temporal patterns:
///
/// **Daily Analysis:**
/// - Daily earthquake counts for time series analysis
/// - Identification of earthquake swarms or sequences
/// - Long-term temporal trends and patterns
///
/// **Frequency Patterns:**
/// - Hourly distribution (0-23): Circadian patterns in earthquake occurrence
/// - Monthly distribution (1-12): Seasonal variations in seismic activity
/// - Weekly patterns: Day-of-week earthquake frequency
///
/// **Applications:**
/// - Temporal correlation analysis
/// - Earthquake forecasting models
/// - Research on triggering mechanisms
/// - Statistical analysis of earthquake cycles
pub struct TemporalPatternsAnalytics {
    date_counts: Arc<RwLock<HashMap<NaiveDate, u32>>>,
    hourly_counts: Arc<RwLock<HashMap<u32, u32>>>,
    monthly_counts: Arc<RwLock<HashMap<u32, u32>>>,
    weekly_counts: Arc<RwLock<HashMap<Weekday, u32>>>,
}

impl TemporalPatternsAnalytics {
    pub fn new() -> Self {
        Self {
            date_counts: Arc::new(RwLock::new(HashMap::new())),
            hourly_counts: Arc::new(RwLock::new(HashMap::new())),
            monthly_counts: Arc::new(RwLock::new(HashMap::new())),
            weekly_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get daily earthquake counts (legacy method for compatibility)
    pub fn get_result(&self) -> Vec<(NaiveDate, u32)> {
        self.get_daily_counts()
    }

    /// Get daily earthquake counts
    pub fn get_daily_counts(&self) -> Vec<(NaiveDate, u32)> {
        let counts = self.date_counts.read();
        let mut result: Vec<_> = counts.iter().map(|(date, count)| (*date, *count)).collect();
        result.sort_by_key(|item| item.0);
        result
    }

    /// Get hourly distribution (0-23 hours)
    pub fn get_hourly_distribution(&self) -> Vec<(u32, u32)> {
        let counts = self.hourly_counts.read();
        let mut result: Vec<_> = counts.iter().map(|(hour, count)| (*hour, *count)).collect();
        result.sort_by_key(|item| item.0);
        result
    }

    /// Get monthly distribution (1-12 months)
    pub fn get_monthly_distribution(&self) -> Vec<(u32, u32)> {
        let counts = self.monthly_counts.read();
        let mut result: Vec<_> = counts
            .iter()
            .map(|(month, count)| (*month, *count))
            .collect();
        result.sort_by_key(|item| item.0);
        result
    }

    /// Get weekly distribution with weekday names
    pub fn get_weekly_distribution(&self) -> Vec<(String, u32)> {
        use chrono::Weekday;
        let counts = self.weekly_counts.read();
        
        // Always return all 7 weekdays, even if some have zero counts
        let all_weekdays = [
            Weekday::Mon,
            Weekday::Tue, 
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ];
        
        all_weekdays
            .iter()
            .map(|weekday| {
                let count = counts.get(weekday).copied().unwrap_or(0);
                (format!("{:?}", weekday), count)
            })
            .collect()
    }
}

impl AnalyticsProcessor for TemporalPatternsAnalytics {
    fn name(&self) -> &'static str {
        "temporal_patterns"
    }

    fn update(&self, event: &SeismicEvent) -> Result<(), PolarsError> {
        let date = event.time.date_naive();
        let hour = event.time.hour();
        let month = event.time.month();
        let weekday = event.time.weekday();

        {
            let mut counts = self.date_counts.write();
            *counts.entry(date).or_insert(0) += 1;
        }

        {
            let mut hourly = self.hourly_counts.write();
            *hourly.entry(hour).or_insert(0) += 1;
        }

        {
            let mut monthly = self.monthly_counts.write();
            *monthly.entry(month).or_insert(0) += 1;
        }

        {
            let mut weekly = self.weekly_counts.write();
            *weekly.entry(weekday).or_insert(0) += 1;
        }

        Ok(())
    }

    fn recompute(&self, dataframe: &LazyFrame) -> Result<(), PolarsError> {
        let result = dataframe.clone().select([col("time")]).collect()?;

        let timestamps = result.column("time")?.datetime()?;
        let mut date_counts = HashMap::new();
        let mut hourly_counts = HashMap::new();
        let mut monthly_counts = HashMap::new();
        let mut weekly_counts = HashMap::new();

        for timestamp_opt in timestamps.iter() {
            if let Some(timestamp) = timestamp_opt {
                let datetime = chrono::DateTime::from_timestamp_nanos(timestamp);
                let date = datetime.date_naive();
                let hour = datetime.hour();
                let month = datetime.month();
                let weekday = datetime.weekday();

                *date_counts.entry(date).or_insert(0) += 1;
                *hourly_counts.entry(hour).or_insert(0) += 1;
                *monthly_counts.entry(month).or_insert(0) += 1;
                *weekly_counts.entry(weekday).or_insert(0) += 1;
            }
        }

        *self.date_counts.write() = date_counts;
        *self.hourly_counts.write() = hourly_counts;
        *self.monthly_counts.write() = monthly_counts;
        *self.weekly_counts.write() = weekly_counts;
        Ok(())
    }

    fn clear(&self) {
        self.date_counts.write().clear();
        self.hourly_counts.write().clear();
        self.monthly_counts.write().clear();
        self.weekly_counts.write().clear();
    }

    fn get_auxiliary_stats(&self, dataframe: &LazyFrame) -> LazyFrame {
        dataframe
            .clone()
            .with_columns([
                col("time").dt().date().alias("date"),
                col("time").dt().hour().alias("hour"),
                col("time").dt().month().alias("month"),
            ])
            .group_by([col("date")])
            .agg([len().alias("daily_count")])
            .sort(["date"], SortMultipleOptions::default())
            .with_columns([lit("Temporal Patterns Analysis").alias("title")])
    }
}

/// Magnitude-depth pairs analytics processor
///
/// This processor collects and analyzes the relationship between earthquake
/// magnitude and depth (hypocenter depth below surface). The magnitude-depth
/// correlation can reveal important seismological insights:
/// - Depth distribution of different magnitude earthquakes
/// - Shallow vs deep earthquake characteristics
/// - Tectonic setting indicators (subduction zones have deep earthquakes)
///
/// The pairs can be used for scatter plots, correlation analysis, and
/// statistical modeling of the magnitude-depth relationship.
pub struct MagnitudeDepthAnalytics {
    pairs: Arc<RwLock<Vec<(f64, f64)>>>,
}

impl MagnitudeDepthAnalytics {
    pub fn new() -> Self {
        Self {
            pairs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_result(&self) -> Vec<(f64, f64)> {
        self.pairs.read().clone()
    }
}

impl AnalyticsProcessor for MagnitudeDepthAnalytics {
    fn name(&self) -> &'static str {
        "magnitude_depth_pairs"
    }

    fn update(&self, event: &SeismicEvent) -> Result<(), PolarsError> {
        let mut pairs = self.pairs.write();
        pairs.push((event.magnitude, event.depth));
        Ok(())
    }

    fn recompute(&self, dataframe: &LazyFrame) -> Result<(), PolarsError> {
        let result = dataframe
            .clone()
            .select([col("mag"), col("depth")])
            .collect()?;

        let magnitudes = result.column("mag")?.f64()?;
        let depths = result.column("depth")?.f64()?;

        let mut pairs = Vec::new();
        for (mag_opt, depth_opt) in magnitudes.iter().zip(depths.iter()) {
            if let (Some(mag), Some(depth)) = (mag_opt, depth_opt) {
                pairs.push((mag, depth));
            }
        }

        *self.pairs.write() = pairs;
        Ok(())
    }

    fn clear(&self) {
        self.pairs.write().clear();
    }

    fn get_auxiliary_stats(&self, dataframe: &LazyFrame) -> LazyFrame {
        dataframe
            .clone()
            .select([
                col("depth").mean().alias("mean_depth"),
                col("depth").median().alias("median_depth"),
                col("depth").std(1).alias("std_depth"),
                col("depth").min().alias("min_depth"),
                col("depth").max().alias("max_depth"),
            ])
            .with_columns([lit("Depth Statistics").alias("title")])
    }
}

/// Geographic hotspots analytics processor - identifies most active regions
///
/// This processor analyzes the spatial distribution of earthquakes to identify
/// geographic areas with high seismic activity. It provides two types of
/// analysis:
///
/// 1. **Region-based analysis**: Groups earthquakes by Flynn region names to
///    identify the most seismically active named regions (e.g., "Southern
///    California", "Japan").
///
/// 2. **Coordinate clustering**: Groups earthquakes into a 0.5-degree grid to
///    create spatial clusters for mapping and visualization. This helps
///    identify hotspots that may not align with named regions.
///
/// Applications include:
/// - Risk assessment for populated areas
/// - Infrastructure planning and building codes
/// - Emergency preparedness planning
/// - Scientific research on fault systems
pub struct GeographicHotspotsAnalytics {
    region_counts: Arc<RwLock<HashMap<String, u32>>>,
    coordinate_clusters: Arc<RwLock<Vec<(f64, f64, u32)>>>, // lat, lon, count
}

impl GeographicHotspotsAnalytics {
    pub fn new() -> Self {
        Self {
            region_counts: Arc::new(RwLock::new(HashMap::new())),
            coordinate_clusters: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_region_hotspots(&self) -> Vec<(String, u32)> {
        let counts = self.region_counts.read();
        let mut result: Vec<_> = counts
            .iter()
            .map(|(region, count)| (region.clone(), *count))
            .collect();
        result.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        result
    }

    pub fn get_coordinate_clusters(&self) -> Vec<(f64, f64, u32)> {
        self.coordinate_clusters.read().clone()
    }
}

impl AnalyticsProcessor for GeographicHotspotsAnalytics {
    fn name(&self) -> &'static str {
        "geographic_hotspots"
    }

    fn update(&self, event: &SeismicEvent) -> Result<(), PolarsError> {
        {
            let mut regions = self.region_counts.write();
            *regions.entry(event.flynn_region.clone()).or_insert(0) += 1;
        }

        let lat_cluster = (event.latitude * 2.0).round() / 2.0;
        let lon_cluster = (event.longitude * 2.0).round() / 2.0;

        {
            let mut clusters = self.coordinate_clusters.write();
            if let Some(existing) = clusters.iter_mut().find(|(lat, lon, _)| {
                (*lat - lat_cluster).abs() < 0.01 && (*lon - lon_cluster).abs() < 0.01
            }) {
                existing.2 += 1;
            } else {
                clusters.push((lat_cluster, lon_cluster, 1));
            }
        }

        Ok(())
    }

    fn recompute(&self, dataframe: &LazyFrame) -> Result<(), PolarsError> {
        let result = dataframe
            .clone()
            .select([col("flynn_region"), col("lat"), col("lon")])
            .collect()?;

        let regions = result.column("flynn_region")?.str()?;
        let lats = result.column("lat")?.f64()?;
        let lons = result.column("lon")?.f64()?;

        let mut region_counts = HashMap::new();
        let mut coordinate_clusters: HashMap<(i32, i32), u32> = HashMap::new();

        for ((region_opt, lat_opt), lon_opt) in regions.iter().zip(lats.iter()).zip(lons.iter()) {
            if let (Some(region), Some(lat), Some(lon)) = (region_opt, lat_opt, lon_opt) {
                *region_counts.entry(region.to_string()).or_insert(0) += 1;

                let lat_key = (lat * 2.0).round() as i32;
                let lon_key = (lon * 2.0).round() as i32;
                *coordinate_clusters.entry((lat_key, lon_key)).or_insert(0) += 1;
            }
        }

        *self.region_counts.write() = region_counts;

        let clusters: Vec<(f64, f64, u32)> = coordinate_clusters
            .into_iter()
            .map(|((lat_key, lon_key), count)| (lat_key as f64 / 2.0, lon_key as f64 / 2.0, count))
            .collect();
        *self.coordinate_clusters.write() = clusters;

        Ok(())
    }

    fn clear(&self) {
        self.region_counts.write().clear();
        self.coordinate_clusters.write().clear();
    }

    fn get_auxiliary_stats(&self, dataframe: &LazyFrame) -> LazyFrame {
        dataframe
            .clone()
            .group_by([col("flynn_region")])
            .agg([
                len().alias("event_count"),
                col("mag").mean().alias("avg_magnitude"),
            ])
            .sort(
                ["event_count"],
                SortMultipleOptions::default().with_order_descending(true),
            )
            .limit(10)
            .with_columns([lit("Geographic Hotspots").alias("title")])
    }
}

/// Gutenberg-Richter law analytics processor - calculates b-value and
/// magnitude-frequency relationship
///
/// This processor implements the fundamental Gutenberg-Richter relationship in
/// seismology: **log₁₀(N) = a - b × M**
///
/// Where:
/// - N = number of earthquakes with magnitude ≥ M
/// - M = magnitude
/// - a = activity rate parameter (log of total earthquake rate)
/// - b = slope parameter (typically ~1.0, indicates stress state)
///
/// **The b-value is crucial for seismic hazard assessment:**
/// - **b < 1.0**: Higher stress environment, more large earthquakes expected
/// - **b > 1.0**: Lower stress environment, more small earthquakes dominate
/// - **b ≈ 1.0**: Typical global average
///
/// The processor uses linear regression on log-transformed data above the
/// magnitude of completeness (Mc = 2.0) to ensure statistical reliability. This
/// is the industry standard method for seismic hazard analysis and earthquake
/// forecasting.
pub struct GutenbergRichterAnalytics {
    magnitude_counts: Arc<RwLock<HashMap<u32, u32>>>, // magnitude * 10 -> count
    b_value: Arc<RwLock<f64>>,
    a_value: Arc<RwLock<f64>>,
    completeness_magnitude: Arc<RwLock<f64>>,
}

impl GutenbergRichterAnalytics {
    pub fn new() -> Self {
        Self {
            magnitude_counts: Arc::new(RwLock::new(HashMap::new())),
            b_value: Arc::new(RwLock::new(1.0)), // Typical b-value around 1.0
            a_value: Arc::new(RwLock::new(0.0)),
            completeness_magnitude: Arc::new(RwLock::new(2.0)),
        }
    }

    pub fn get_b_value(&self) -> f64 {
        *self.b_value.read()
    }

    pub fn get_a_value(&self) -> f64 {
        *self.a_value.read()
    }

    pub fn get_completeness_magnitude(&self) -> f64 {
        *self.completeness_magnitude.read()
    }

    pub fn get_magnitude_frequency_data(&self) -> Vec<(f64, u32, u32)> {
        let counts = self.magnitude_counts.read();
        let mut result = Vec::new();

        let mut sorted_mags: Vec<_> = counts.keys().collect();
        sorted_mags.sort();

        for &mag_key in &sorted_mags {
            let magnitude = *mag_key as f64 / 10.0;
            let count = *counts.get(mag_key).unwrap_or(&0);

            let cumulative_count: u32 = sorted_mags
                .iter()
                .filter(|&&m| m >= mag_key)
                .map(|&m| counts.get(m).unwrap_or(&0))
                .sum();

            result.push((magnitude, count, cumulative_count));
        }

        result
    }

    fn calculate_b_value(&self) {
        let counts = self.magnitude_counts.read();
        if counts.len() < 3 {
            return; // Need at least 3 data points
        }

        let completeness_mag = *self.completeness_magnitude.read();
        let completeness_key = (completeness_mag * 10.0) as u32;

        let valid_data: Vec<(f64, f64)> = counts
            .iter()
            .filter(|(&mag_key, &count)| mag_key >= completeness_key && count > 0)
            .map(|(&mag_key, &count)| {
                let magnitude = mag_key as f64 / 10.0;
                let log_count = (count as f64).ln();
                (magnitude, log_count)
            })
            .collect();

        if valid_data.len() < 3 {
            return;
        }

        let n = valid_data.len() as f64;
        let sum_m: f64 = valid_data.iter().map(|(m, _)| m).sum();
        let sum_log_n: f64 = valid_data.iter().map(|(_, log_n)| log_n).sum();
        let sum_m_log_n: f64 = valid_data.iter().map(|(m, log_n)| m * log_n).sum();
        let sum_m_squared: f64 = valid_data.iter().map(|(m, _)| m * m).sum();

        let b_value = (n * sum_m_log_n - sum_m * sum_log_n) / (sum_m * sum_m - n * sum_m_squared);
        let a_value = (sum_log_n - b_value * sum_m) / n;

        *self.b_value.write() = -b_value; // Negative because of the relationship
        *self.a_value.write() = a_value;
    }
}

impl AnalyticsProcessor for GutenbergRichterAnalytics {
    fn name(&self) -> &'static str {
        "gutenberg_richter"
    }

    fn update(&self, event: &SeismicEvent) -> Result<(), PolarsError> {
        let mag_key = (event.magnitude * 10.0) as u32;
        {
            let mut counts = self.magnitude_counts.write();
            *counts.entry(mag_key).or_insert(0) += 1;
        }

        if self.magnitude_counts.read().values().sum::<u32>() % 100 == 0 {
            self.calculate_b_value();
        }

        Ok(())
    }

    fn recompute(&self, dataframe: &LazyFrame) -> Result<(), PolarsError> {
        let result = dataframe.clone().select([col("mag")]).collect()?;

        let magnitudes = result.column("mag")?.f64()?;
        let mut magnitude_counts = HashMap::new();

        for mag_opt in magnitudes.iter() {
            if let Some(mag) = mag_opt {
                let mag_key = (mag * 10.0) as u32;
                *magnitude_counts.entry(mag_key).or_insert(0) += 1;
            }
        }

        *self.magnitude_counts.write() = magnitude_counts;
        self.calculate_b_value();
        Ok(())
    }

    fn clear(&self) {
        self.magnitude_counts.write().clear();
        *self.b_value.write() = 1.0;
        *self.a_value.write() = 0.0;
        *self.completeness_magnitude.write() = 2.0;
    }

    fn get_auxiliary_stats(&self, dataframe: &LazyFrame) -> LazyFrame {
        let b_val = self.get_b_value();
        let a_val = self.get_a_value();
        let mc = self.get_completeness_magnitude();

        dataframe
            .clone()
            .select([
                lit(b_val).alias("b_value"),
                lit(a_val).alias("a_value"),
                lit(mc).alias("completeness_magnitude"),
                col("mag").count().alias("total_events"),
            ])
            .with_columns([lit("Gutenberg-Richter Analysis").alias("title")])
    }
}

/// Risk assessment analytics processor - calculates probabilities and energy
/// release
///
/// This processor provides quantitative risk assessment metrics for earthquake
/// hazards:
///
/// **1. Probability Calculations:**
/// Uses Poisson statistics to calculate the probability of earthquakes above
/// certain magnitude thresholds occurring within specified time windows:
/// - P(M≥5.0 in next 30 days)
/// - P(M≥6.0 in next 365 days)
/// - P(M≥7.0 in next 365 days)
///
/// Formula: P(X≥1) = 1 - e^(-λt), where λ = rate per day, t = time period
///
/// **2. Seismic Energy Release:**
/// Calculates total energy released using the relationship:
/// **log₁₀(E) = 11.8 + 1.5 × M** (energy in Joules)
///
/// **Applications:**
/// - Emergency preparedness planning
/// - Building code development
/// - Public safety communications
/// - Scientific research on earthquake cycles
///
/// The calculations are based on historical earthquake rates and assume
/// stationary seismicity (constant rate over time).
pub struct RiskAssessmentAnalytics {
    total_events: Arc<RwLock<u32>>,
    time_span_days: Arc<RwLock<f64>>,
    magnitude_counts: Arc<RwLock<HashMap<u32, u32>>>,
    total_energy_joules: Arc<RwLock<f64>>,
}

impl RiskAssessmentAnalytics {
    pub fn new() -> Self {
        Self {
            total_events: Arc::new(RwLock::new(0)),
            time_span_days: Arc::new(RwLock::new(1.0)),
            magnitude_counts: Arc::new(RwLock::new(HashMap::new())),
            total_energy_joules: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Calculate probability of magnitude >= threshold in next N days
    pub fn probability_magnitude_in_days(&self, magnitude_threshold: f64, days: f64) -> f64 {
        let counts = self.magnitude_counts.read();
        let time_span = *self.time_span_days.read();
        let threshold_key = (magnitude_threshold * 10.0) as u32;

        let events_above_threshold: u32 = counts
            .iter()
            .filter(|(&mag_key, _)| mag_key >= threshold_key)
            .map(|(_, &count)| count)
            .sum();

        if time_span <= 0.0 {
            return 0.0;
        }

        let rate_per_day = events_above_threshold as f64 / time_span;

        let lambda_t = rate_per_day * days;
        1.0 - (-lambda_t).exp()
    }

    /// Calculate total seismic energy released (in Joules)
    pub fn get_total_energy(&self) -> f64 {
        *self.total_energy_joules.read()
    }

    /// Convert magnitude to energy (Joules) using: log10(E) = 11.8 + 1.5*M
    fn magnitude_to_energy(magnitude: f64) -> f64 {
        let log_energy = 11.8 + 1.5 * magnitude;
        10_f64.powf(log_energy)
    }

    pub fn get_risk_metrics(&self) -> (f64, f64, f64, f64) {
        let prob_5_30days = self.probability_magnitude_in_days(5.0, 30.0);
        let prob_6_365days = self.probability_magnitude_in_days(6.0, 365.0);
        let prob_7_365days = self.probability_magnitude_in_days(7.0, 365.0);
        let total_energy = self.get_total_energy();

        (prob_5_30days, prob_6_365days, prob_7_365days, total_energy)
    }
}

impl AnalyticsProcessor for RiskAssessmentAnalytics {
    fn name(&self) -> &'static str {
        "risk_assessment"
    }

    fn update(&self, event: &SeismicEvent) -> Result<(), PolarsError> {
        {
            let mut total = self.total_events.write();
            *total += 1;
        }

        let mag_key = (event.magnitude * 10.0) as u32;
        {
            let mut counts = self.magnitude_counts.write();
            *counts.entry(mag_key).or_insert(0) += 1;
        }

        let energy = Self::magnitude_to_energy(event.magnitude);
        {
            let mut total_energy = self.total_energy_joules.write();
            *total_energy += energy;
        }

        Ok(())
    }

    fn recompute(&self, dataframe: &LazyFrame) -> Result<(), PolarsError> {
        let result = dataframe
            .clone()
            .select([col("mag"), col("time")])
            .collect()?;

        let magnitudes = result.column("mag")?.f64()?;
        let timestamps = result.column("time")?.datetime()?;

        let mut magnitude_counts = HashMap::new();
        let mut total_energy = 0.0;
        let mut min_time = i64::MAX;
        let mut max_time = i64::MIN;

        for (mag_opt, time_opt) in magnitudes.iter().zip(timestamps.iter()) {
            if let (Some(mag), Some(time)) = (mag_opt, time_opt) {
                let mag_key = (mag * 10.0) as u32;
                *magnitude_counts.entry(mag_key).or_insert(0) += 1;

                total_energy += Self::magnitude_to_energy(mag);

                min_time = min_time.min(time);
                max_time = max_time.max(time);
            }
        }

        let time_span_days = if min_time < max_time {
            (max_time - min_time) as f64 / (1_000_000_000.0 * 86400.0) // nanoseconds to days
        } else {
            1.0
        };

        *self.total_events.write() = magnitudes.len() as u32;
        *self.magnitude_counts.write() = magnitude_counts;
        *self.total_energy_joules.write() = total_energy;
        *self.time_span_days.write() = time_span_days;

        Ok(())
    }

    fn clear(&self) {
        *self.total_events.write() = 0;
        *self.time_span_days.write() = 1.0;
        self.magnitude_counts.write().clear();
        *self.total_energy_joules.write() = 0.0;
    }

    fn get_auxiliary_stats(&self, dataframe: &LazyFrame) -> LazyFrame {
        let (prob_5_30, prob_6_365, prob_7_365, total_energy) = self.get_risk_metrics();

        dataframe
            .clone()
            .select([
                lit(prob_5_30).alias("prob_mag5_30days"),
                lit(prob_6_365).alias("prob_mag6_365days"),
                lit(prob_7_365).alias("prob_mag7_365days"),
                lit(total_energy).alias("total_energy_joules"),
                col("mag").count().alias("total_events"),
            ])
            .with_columns([lit("Risk Assessment").alias("title")])
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;
    use crate::seismic::SeismicEvent;
    use crate::test_utils::create_test_event_with_params;

    #[test]
    fn test_magnitude_distribution_analytics_comprehensive() {
        let processor = MagnitudeDistributionAnalytics::new();

        assert_eq!(processor.get_result().unwrap().len(), 0);

        assert_eq!(processor.name(), "magnitude_distribution");

        processor.clear();
        assert_eq!(processor.get_result().unwrap().len(), 0);

        let magnitudes = vec![1.5, 2.0, 2.1, 2.3, 3.0, 3.1, 4.5, 4.7];
        for (i, mag) in magnitudes.iter().enumerate() {
            let mut event = SeismicEvent::test_event();
            event.id = format!("test_{}", i);
            event.magnitude = *mag;
            processor.update(&event).unwrap();
        }

        let distribution = processor.get_result().unwrap();
        assert!(!distribution.is_empty());

        let bucket_1_4 = distribution.iter().find(|(mag, _)| mag == "1.4");
        assert!(bucket_1_4.is_some());
        assert_eq!(bucket_1_4.unwrap().1, 1); // 1.5

        let bucket_2_0 = distribution.iter().find(|(mag, _)| mag == "2");
        assert!(bucket_2_0.is_some());
        assert_eq!(bucket_2_0.unwrap().1, 2); // 2.0, 2.1

        let bucket_2_2 = distribution.iter().find(|(mag, _)| mag == "2.2");
        assert!(bucket_2_2.is_some());
        assert_eq!(bucket_2_2.unwrap().1, 1); // 2.3

        let mags: Vec<f32> = distribution
            .iter()
            .map(|(mag, _)| mag.parse::<f32>().unwrap())
            .collect();
        let mut sorted_mags = mags.clone();
        sorted_mags.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(mags, sorted_mags);
    }

    #[test]
    fn test_temporal_patterns_analytics_comprehensive() {
        let processor = TemporalPatternsAnalytics::new();

        assert_eq!(processor.get_result().len(), 0);
        assert_eq!(processor.get_daily_counts().len(), 0);
        assert_eq!(processor.get_hourly_distribution().len(), 0);
        assert_eq!(processor.get_monthly_distribution().len(), 0);
        assert_eq!(processor.get_weekly_distribution().len(), 7);

        assert_eq!(processor.name(), "temporal_patterns");

        let base_time = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let events = vec![
            create_test_event_with_params("1", 2.0, 10.0, 35.0, -120.0, base_time, "California"),
            create_test_event_with_params(
                "2",
                2.1,
                15.0,
                35.1,
                -120.1,
                base_time + chrono::TimeDelta::hours(5),
                "California",
            ),
            create_test_event_with_params(
                "3",
                2.2,
                20.0,
                35.2,
                -120.2,
                base_time + chrono::TimeDelta::hours(12),
                "California",
            ),
            create_test_event_with_params(
                "4",
                3.0,
                25.0,
                36.0,
                -121.0,
                base_time + chrono::TimeDelta::days(1),
                "California",
            ),
            create_test_event_with_params(
                "5",
                3.5,
                30.0,
                37.0,
                -122.0,
                base_time + chrono::TimeDelta::days(32),
                "California",
            ),
            create_test_event_with_params(
                "6",
                4.0,
                35.0,
                38.0,
                -123.0,
                base_time + chrono::TimeDelta::days(3),
                "California",
            ),
        ];

        for event in &events {
            processor.update(event).unwrap();
        }

        let daily_counts = processor.get_daily_counts();
        assert!(!daily_counts.is_empty());
        assert!(daily_counts.len() >= 3); // At least 3 different dates

        let hourly_dist = processor.get_hourly_distribution();
        assert!(!hourly_dist.is_empty());

        let total_events: u32 = hourly_dist.iter().map(|(_, count)| count).sum();
        assert_eq!(total_events, 6); // Total number of events

        let monthly_dist = processor.get_monthly_distribution();
        assert!(!monthly_dist.is_empty());
        assert!(monthly_dist.len() >= 2); // January and February

        let weekly_dist = processor.get_weekly_distribution();
        assert!(!weekly_dist.is_empty());

        for (weekday_name, _) in &weekly_dist {
            assert!(matches!(
                weekday_name.as_str(),
                "Mon" | "Tue" | "Wed" | "Thu" | "Fri" | "Sat" | "Sun"
            ));
        }

        processor.clear();
        assert_eq!(processor.get_daily_counts().len(), 0);
        assert_eq!(processor.get_hourly_distribution().len(), 0);
        assert_eq!(processor.get_monthly_distribution().len(), 0);
        assert_eq!(processor.get_weekly_distribution().len(), 0);
    }

    #[test]
    fn test_magnitude_depth_analytics_comprehensive() {
        let processor = MagnitudeDepthAnalytics::new();

        assert_eq!(processor.get_result().len(), 0);
        assert_eq!(processor.name(), "magnitude_depth_pairs");

        let test_pairs = vec![
            (2.0, 5.0),
            (3.5, 15.0),
            (4.0, 25.0),
            (5.5, 35.0),
            (6.0, 45.0),
        ];

        for (i, (mag, depth)) in test_pairs.iter().enumerate() {
            let mut event = SeismicEvent::test_event();
            event.id = format!("test_{}", i);
            event.magnitude = *mag;
            event.depth = *depth;
            processor.update(&event).unwrap();
        }

        let pairs = processor.get_result();
        assert_eq!(pairs.len(), 5);

        for (expected_mag, expected_depth) in test_pairs {
            assert!(pairs
                .iter()
                .any(|(mag, depth)| (*mag - expected_mag).abs() < 0.001
                    && (*depth - expected_depth).abs() < 0.001));
        }

        processor.clear();
        assert_eq!(processor.get_result().len(), 0);
    }

    #[test]
    fn test_geographic_hotspots_analytics_comprehensive() {
        let processor = GeographicHotspotsAnalytics::new();

        assert_eq!(processor.get_region_hotspots().len(), 0);
        assert_eq!(processor.get_coordinate_clusters().len(), 0);
        assert_eq!(processor.name(), "geographic_hotspots");

        let events = vec![
            create_test_event_with_params("1", 2.0, 10.0, 35.0, -120.0, Utc::now(), "California"),
            create_test_event_with_params("2", 2.1, 15.0, 35.1, -120.1, Utc::now(), "California"), /* Same region, close coordinates */
            create_test_event_with_params("3", 3.0, 20.0, 40.0, -125.0, Utc::now(), "Oregon"), /* Different region */
            create_test_event_with_params("4", 3.5, 25.0, 35.2, -120.2, Utc::now(), "California"), /* Same region, close coordinates */
            create_test_event_with_params("5", 4.0, 30.0, 45.0, -130.0, Utc::now(), "Alaska"), /* Different region, far coordinates */
        ];

        for event in &events {
            processor.update(event).unwrap();
        }

        let region_hotspots = processor.get_region_hotspots();
        assert!(!region_hotspots.is_empty());

        let california = region_hotspots
            .iter()
            .find(|(region, _)| region == "California");
        assert!(california.is_some());
        assert_eq!(california.unwrap().1, 3);

        for i in 1..region_hotspots.len() {
            assert!(region_hotspots[i - 1].1 >= region_hotspots[i].1);
        }

        let clusters = processor.get_coordinate_clusters();
        assert!(!clusters.is_empty());

        let california_cluster = clusters
            .iter()
            .find(|(lat, lon, _)| (*lat - 35.0).abs() < 1.0 && (*lon - (-120.0)).abs() < 1.0);
        assert!(california_cluster.is_some());
        assert!(california_cluster.unwrap().2 >= 3);

        processor.clear();
        assert_eq!(processor.get_region_hotspots().len(), 0);
        assert_eq!(processor.get_coordinate_clusters().len(), 0);
    }

    #[test]
    fn test_gutenberg_richter_analytics_comprehensive() {
        let processor = GutenbergRichterAnalytics::new();

        assert_eq!(processor.name(), "gutenberg_richter");
        assert_eq!(processor.get_b_value(), 1.0); // Default b-value
        assert_eq!(processor.get_a_value(), 0.0); // Default a-value
        assert_eq!(processor.get_completeness_magnitude(), 2.0);
        assert_eq!(processor.get_magnitude_frequency_data().len(), 0);

        let magnitudes = vec![
            2.0, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, // Many small earthquakes
            3.0, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, // Fewer medium earthquakes
            4.0, 4.1, 4.2, 4.3, 4.4, // Even fewer larger earthquakes
            5.0, 5.1, 5.2, // Very few large earthquakes
            6.0, // One very large earthquake
        ];

        for (i, mag) in magnitudes.iter().enumerate() {
            let mut event = SeismicEvent::test_event();
            event.id = format!("test_{}", i);
            event.magnitude = *mag;
            processor.update(&event).unwrap();
        }

        let freq_data = processor.get_magnitude_frequency_data();
        assert!(!freq_data.is_empty());

        let mut prev_cumulative = u32::MAX;
        for (magnitude, _count, cumulative) in freq_data {
            if magnitude >= 2.0 {
                assert!(cumulative <= prev_cumulative);
                prev_cumulative = cumulative;
            }
        }

        let b_value = processor.get_b_value();
        assert!(b_value > 0.0);
        assert!(b_value < 5.0); // Reasonable range

        processor.clear();
        assert_eq!(processor.get_b_value(), 1.0);
        assert_eq!(processor.get_a_value(), 0.0);
        assert_eq!(processor.get_magnitude_frequency_data().len(), 0);
    }

    #[test]
    fn test_risk_assessment_analytics_comprehensive() {
        let processor = RiskAssessmentAnalytics::new();

        assert_eq!(processor.name(), "risk_assessment");
        assert_eq!(processor.get_total_energy(), 0.0);
        let (prob_5_30, prob_6_365, prob_7_365, total_energy) = processor.get_risk_metrics();
        assert_eq!(prob_5_30, 0.0);
        assert_eq!(prob_6_365, 0.0);
        assert_eq!(prob_7_365, 0.0);
        assert_eq!(total_energy, 0.0);

        let magnitudes = vec![2.0, 3.0, 4.0, 5.0, 5.5, 6.0, 6.5];
        let base_time = Utc::now();

        for (i, mag) in magnitudes.iter().enumerate() {
            let event_time = base_time + chrono::TimeDelta::days(i as i64);
            let event = create_test_event_with_params(
                &format!("test_{}", i),
                *mag,
                10.0,
                35.0,
                -120.0,
                event_time,
                "California",
            );
            processor.update(&event).unwrap();
        }

        let total_energy = processor.get_total_energy();
        assert!(total_energy > 0.0);

        let energy_2_0 = RiskAssessmentAnalytics::magnitude_to_energy(2.0);
        let energy_6_5 = RiskAssessmentAnalytics::magnitude_to_energy(6.5);
        assert!(energy_6_5 > energy_2_0 * 1000.0); // Much more energy

        let prob_5_0_30days = processor.probability_magnitude_in_days(5.0, 30.0);
        let prob_6_0_365days = processor.probability_magnitude_in_days(6.0, 365.0);
        let prob_7_0_365days = processor.probability_magnitude_in_days(7.0, 365.0);

        assert!(prob_5_0_30days >= 0.0 && prob_5_0_30days <= 1.0);
        assert!(prob_6_0_365days >= 0.0 && prob_6_0_365days <= 1.0);
        assert!(prob_7_0_365days >= 0.0 && prob_7_0_365days <= 1.0);

        assert!(prob_7_0_365days <= prob_6_0_365days);

        let (prob_5_30, prob_6_365, prob_7_365, energy) = processor.get_risk_metrics();
        assert_eq!(prob_5_30, prob_5_0_30days);
        assert_eq!(prob_6_365, prob_6_0_365days);
        assert_eq!(prob_7_365, prob_7_0_365days);
        assert_eq!(energy, total_energy);

        processor.clear();
        assert_eq!(processor.get_total_energy(), 0.0);
        let (prob_5_30, prob_6_365, prob_7_365, total_energy) = processor.get_risk_metrics();
        assert_eq!(prob_5_30, 0.0);
        assert_eq!(prob_6_365, 0.0);
        assert_eq!(prob_7_365, 0.0);
        assert_eq!(total_energy, 0.0);
    }

    #[test]
    fn test_analytics_processor_trait_methods() {
        let processors: Vec<Box<dyn AnalyticsProcessor>> = vec![
            Box::new(MagnitudeDistributionAnalytics::new()),
            Box::new(TemporalPatternsAnalytics::new()),
            Box::new(MagnitudeDepthAnalytics::new()),
            Box::new(GeographicHotspotsAnalytics::new()),
            Box::new(GutenbergRichterAnalytics::new()),
            Box::new(RiskAssessmentAnalytics::new()),
        ];

        let expected_names = vec![
            "magnitude_distribution",
            "temporal_patterns",
            "magnitude_depth_pairs",
            "geographic_hotspots",
            "gutenberg_richter",
            "risk_assessment",
        ];

        for (processor, expected_name) in processors.iter().zip(expected_names.iter()) {
            assert_eq!(processor.name(), *expected_name);

            let event = SeismicEvent::test_event();
            assert!(processor.update(&event).is_ok());

            processor.clear();
        }
    }

    #[test]
    fn test_weekday_ordering() {
        let processor = TemporalPatternsAnalytics::new();

        let base_time = DateTime::parse_from_rfc3339("2024-01-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc); // Monday

        for i in 0..7 {
            let event_time = base_time + chrono::TimeDelta::days(i);
            let event = create_test_event_with_params(
                &format!("test_{}", i),
                2.0,
                10.0,
                35.0,
                -120.0,
                event_time,
                "California",
            );
            processor.update(&event).unwrap();
        }

        let weekly_dist = processor.get_weekly_distribution();
        assert_eq!(weekly_dist.len(), 7);

        let expected_order = vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        for (i, (weekday_name, _)) in weekly_dist.iter().enumerate() {
            assert_eq!(weekday_name, expected_order[i]);
        }

        for (_, count) in weekly_dist {
            assert_eq!(count, 1);
        }
    }

    #[test]
    fn test_coordinate_clustering() {
        let processor = GeographicHotspotsAnalytics::new();

        let events = vec![
            create_test_event_with_params("1", 2.0, 10.0, 35.0, -120.0, Utc::now(), "California"),
            create_test_event_with_params("2", 2.1, 15.0, 35.1, -120.1, Utc::now(), "California"),
            create_test_event_with_params("3", 2.2, 20.0, 35.2, -120.2, Utc::now(), "California"),
            create_test_event_with_params("4", 3.0, 25.0, 40.0, -125.0, Utc::now(), "Oregon"),
            create_test_event_with_params("5", 3.1, 30.0, 40.1, -125.1, Utc::now(), "Oregon"),
            create_test_event_with_params("6", 4.0, 35.0, 50.0, -130.0, Utc::now(), "Alaska"),
        ];

        for event in &events {
            processor.update(event).unwrap();
        }

        let clusters = processor.get_coordinate_clusters();
        assert!(clusters.len() >= 3);

        let california_cluster = clusters.iter().find(|(lat, lon, count)| {
            (*lat - 35.0).abs() < 1.0 && (*lon - (-120.0)).abs() < 1.0 && *count == 3
        });
        assert!(california_cluster.is_some());

        let oregon_cluster = clusters.iter().find(|(lat, lon, count)| {
            (*lat - 40.0).abs() < 1.0 && (*lon - (-125.0)).abs() < 1.0 && *count == 2
        });
        assert!(oregon_cluster.is_some());

        let alaska_cluster = clusters.iter().find(|(lat, lon, count)| {
            (*lat - 50.0).abs() < 1.0 && (*lon - (-130.0)).abs() < 1.0 && *count == 1
        });
        assert!(alaska_cluster.is_some());
    }

    #[test]
    fn test_magnitude_energy_conversion() {
        let test_cases = vec![
            (2.0, 11.8 + 1.5 * 2.0), // log10(E) = 14.8
            (4.0, 11.8 + 1.5 * 4.0), // log10(E) = 17.8
            (6.0, 11.8 + 1.5 * 6.0), // log10(E) = 20.8
            (8.0, 11.8 + 1.5 * 8.0), // log10(E) = 23.8
        ];

        for (magnitude, expected_log_energy) in test_cases {
            let energy = RiskAssessmentAnalytics::magnitude_to_energy(magnitude);
            let log_energy = energy.log10();

            assert!((log_energy - expected_log_energy).abs() < 0.001);

            assert!(energy > 0.0);
        }

        let energy_4 = RiskAssessmentAnalytics::magnitude_to_energy(4.0);
        let energy_5 = RiskAssessmentAnalytics::magnitude_to_energy(5.0);
        let energy_6 = RiskAssessmentAnalytics::magnitude_to_energy(6.0);

        let ratio_4_to_5 = energy_5 / energy_4;
        let ratio_5_to_6 = energy_6 / energy_5;

        assert!((ratio_4_to_5 - 31.6).abs() < 1.0);
        assert!((ratio_5_to_6 - 31.6).abs() < 1.0);
    }
}
