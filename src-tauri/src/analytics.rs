use chrono::NaiveDate;

use crate::AppState;

pub mod incremental;
mod processors;

/// Get magnitude distribution using incremental analytics
pub(crate) fn get_magnitude_distribution_internal(
    state: &AppState,
) -> Result<Vec<(String, u32)>, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    state.get_analytics().get_magnitude_distribution()
}

/// Get count by date using incremental analytics
pub(crate) fn get_count_by_year_internal(
    state: &AppState,
) -> Result<Vec<(NaiveDate, u32)>, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    Ok(state.get_analytics().get_count_by_date())
}

/// Get magnitude-depth pairs using incremental analytics
pub(crate) fn get_mag_depth_pairs_internal(state: &AppState) -> Result<Vec<(f64, f64)>, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    Ok(state.get_analytics().get_mag_depth_pairs())
}

/// Get advanced analytics using Polars
pub(crate) fn get_advanced_analytics_internal(
    state: &AppState,
) -> Result<serde_json::Value, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    match state.get_analytics().get_advanced_analytics() {
        Ok(analytics) => analytics.to_json().map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Get hourly frequency distribution
pub(crate) fn get_hourly_frequency_internal(state: &AppState) -> Result<Vec<(u32, u32)>, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    Ok(state.get_analytics().get_hourly_frequency())
}

/// Get monthly frequency distribution
pub(crate) fn get_monthly_frequency_internal(state: &AppState) -> Result<Vec<(u32, u32)>, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    Ok(state.get_analytics().get_monthly_frequency())
}

/// Get geographic hotspots by region
pub(crate) fn get_region_hotspots_internal(state: &AppState) -> Result<Vec<(String, u32)>, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    Ok(state.get_analytics().get_region_hotspots())
}

/// Get coordinate clusters for mapping
pub(crate) fn get_coordinate_clusters_internal(
    state: &AppState,
) -> Result<Vec<(f64, f64, u32)>, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    Ok(state.get_analytics().get_coordinate_clusters())
}

/// Get Gutenberg-Richter b-value
pub(crate) fn get_b_value_internal(state: &AppState) -> Result<f64, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    Ok(state.get_analytics().get_b_value())
}

/// Get magnitude-frequency relationship data
pub(crate) fn get_magnitude_frequency_data_internal(
    state: &AppState,
) -> Result<Vec<(f64, u32, u32)>, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    Ok(state.get_analytics().get_magnitude_frequency_data())
}

/// Get risk assessment metrics
pub(crate) fn get_risk_metrics_internal(state: &AppState) -> Result<(f64, f64, f64, f64), String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    Ok(state.get_analytics().get_risk_metrics())
}

/// Get total seismic energy released
pub(crate) fn get_total_energy_internal(state: &AppState) -> Result<f64, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    Ok(state.get_analytics().get_total_energy())
}

/// Get weekly frequency distribution with weekday names
pub(crate) fn get_weekly_frequency_internal(
    state: &AppState,
) -> Result<Vec<(String, u32)>, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
    Ok(state.get_analytics().get_weekly_frequency())
}

#[cfg(test)]
mod test {
    use std::sync::Mutex;

    use super::*;
    use crate::seismic::SeismicEvent;
    use crate::state::SeismicData;

    fn empty_state() -> AppState {
        Mutex::new(SeismicData::new())
    }

    fn state_with_one_entry() -> AppState {
        let state = empty_state();
        let mut guard = state.lock().unwrap();
        guard
            .add_or_update_event(SeismicEvent::test_event())
            .unwrap();
        drop(guard);
        state
    }

    fn state_with_n_entries(size: usize) -> AppState {
        state_with_n_entries_func(size, |index| {
            let mut event = SeismicEvent::test_event();
            event.id.push_str(&index.to_string());
            event
        })
    }

    fn state_with_n_entries_func(size: usize, create: fn(usize) -> SeismicEvent) -> AppState {
        let state = empty_state();
        let mut guard = state.lock().unwrap();
        let events: Vec<_> = (0..size).map(create).collect();
        guard.add_events(events).unwrap();
        drop(guard);
        state
    }

    #[test]
    fn test_empty_count_by_year() {
        let state = empty_state();

        let response = get_count_by_year_internal(&state).unwrap();

        assert_eq!(response.len(), 0);
    }

    #[test]
    fn test_single_count_by_year() {
        let state = state_with_one_entry();

        let response = get_count_by_year_internal(&state).unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(
            response[0],
            (NaiveDate::from_ymd_opt(2024, 12, 10).unwrap(), 1)
        );
    }

    #[test]
    fn test_multiple_count_by_year() {
        let state = state_with_n_entries(3);

        let response = get_count_by_year_internal(&state).unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(
            response[0],
            (NaiveDate::from_ymd_opt(2024, 12, 10).unwrap(), 3)
        );
    }

    #[test]
    fn test_multiple_count_by_years() {
        let state = state_with_n_entries_func(5, |index| {
            let mut event = SeismicEvent::test_event();
            event.id.push_str(&index.to_string());
            let time_delta = chrono::TimeDelta::days(index.div_euclid(2) as i64);
            event.time = event.time + time_delta;
            event.last_update = event.last_update + time_delta;
            event
        });
        let start_date = NaiveDate::from_ymd_opt(2024, 12, 10).unwrap();

        let response = get_count_by_year_internal(&state).unwrap();

        assert_eq!(response.len(), 3);
        assert_eq!(response[0], (start_date, 2));
        assert_eq!(response[2], (start_date + chrono::TimeDelta::days(2), 1));
    }

    #[test]
    fn test_no_magnitudes() {
        let state = empty_state();

        let response = get_magnitude_distribution_internal(&state).unwrap();

        assert_eq!(response.len(), 0);
    }

    #[test]
    fn test_one_magnitude() {
        let state = state_with_one_entry();

        let response = get_magnitude_distribution_internal(&state).unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].1, 1);
    }

    #[test]
    fn test_many_magnitudes() {
        let state = state_with_n_entries(4);

        let response = get_magnitude_distribution_internal(&state).unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].1, 4);
    }

    #[test]
    fn test_many_magnitudes_bucketing() {
        let state = state_with_n_entries_func(7, |index| {
            let mut event = SeismicEvent::test_event();
            event.id.push_str(&index.to_string());
            event.magnitude += 0.1 * (index as f64);
            event
        });

        let response = get_magnitude_distribution_internal(&state).unwrap();

        assert_eq!(response.len(), 4);
        assert_eq!(response[0].1, 2);
        assert_eq!(response[2].0, "2.4");
        assert_eq!(response[3].1, 1);
    }

    #[test]
    fn test_advanced_analytics() {
        let state = state_with_n_entries(10);

        let result = get_advanced_analytics_internal(&state);

        assert!(result.is_ok());
        let analytics = result.unwrap();
        assert!(analytics.is_object());

        let obj = analytics.as_object().unwrap();
        assert!(obj.contains_key("stats"));

        let stats = obj.get("stats").unwrap().as_array().unwrap();
        assert!(stats.len() >= 4);

        for stat in stats {
            let stat_obj = stat.as_object().unwrap();
            assert!(stat_obj.contains_key("title"));
            assert!(stat_obj.contains_key("data"));
        }

        let titles: Vec<String> = stats
            .iter()
            .map(|s| {
                s.as_object()
                    .unwrap()
                    .get("title")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect();

        assert!(titles.contains(&"Magnitude Statistics".to_string()));
        assert!(titles.contains(&"Temporal Patterns Analysis".to_string()));
        assert!(titles.contains(&"Depth Statistics".to_string()));
        assert!(titles.contains(&"Regional Analysis".to_string()));
    }

    #[test]
    fn test_new_analytics() {
        let state = state_with_n_entries(10);

        let hourly_freq = get_hourly_frequency_internal(&state).unwrap();
        assert!(!hourly_freq.is_empty());

        let monthly_freq = get_monthly_frequency_internal(&state).unwrap();
        assert!(!monthly_freq.is_empty());

        let region_hotspots = get_region_hotspots_internal(&state).unwrap();
        assert!(!region_hotspots.is_empty());

        let coordinate_clusters = get_coordinate_clusters_internal(&state).unwrap();
        assert!(!coordinate_clusters.is_empty());

        let b_value = get_b_value_internal(&state).unwrap();
        assert!(b_value > 0.0);

        let mag_freq_data = get_magnitude_frequency_data_internal(&state).unwrap();
        assert!(!mag_freq_data.is_empty());

        let (prob_5_30, prob_6_365, prob_7_365, total_energy) =
            get_risk_metrics_internal(&state).unwrap();
        assert!(prob_5_30 >= 0.0 && prob_5_30 <= 1.0);
        assert!(prob_6_365 >= 0.0 && prob_6_365 <= 1.0);
        assert!(prob_7_365 >= 0.0 && prob_7_365 <= 1.0);
        assert!(total_energy > 0.0);

        let energy = get_total_energy_internal(&state).unwrap();
        assert_eq!(energy, total_energy);
    }

    #[test]
    fn test_weekday_functionality() {
        let state = state_with_n_entries(10);

        let weekly_freq = get_weekly_frequency_internal(&state).unwrap();
        assert!(!weekly_freq.is_empty());

        for (weekday_name, _count) in &weekly_freq {
            assert!(matches!(
                weekday_name.as_str(),
                "Mon" | "Tue" | "Wed" | "Thu" | "Fri" | "Sat" | "Sun"
            ));
        }

        let total_named: u32 = weekly_freq.iter().map(|(_, count)| count).sum();
        assert_eq!(total_named, 10);
    }
}
