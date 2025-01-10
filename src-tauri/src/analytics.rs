use chrono::{Datelike, NaiveDate};
use itertools::Itertools;

use crate::AppState;

pub(crate) fn get_magnitude_distribution_internal(state: &AppState) -> Vec<(String, u32)> {
    let state = state.lock().unwrap();

    let magnitudes = state.run_on_events(|event| event.magnitude);

    let boxed = magnitudes
        .iter()
        .map(|mag| {
            let mag = (mag * 10f64) as u32;
            mag - (mag % 2)
        })
        .sorted()
        .chunk_by(|mag| *mag)
        .into_iter()
        .sorted_by_key(|(mag, _)| *mag)
        .map(|(mag, chunk)| ((mag as f32 / 10f32).to_string(), chunk.count() as u32))
        .collect();

    boxed
}

pub(crate) fn get_count_by_year_internal(state: &AppState) -> Vec<(NaiveDate, u32)> {
    let state = state.lock().unwrap();

    state
        .run_on_events(|event| event.time)
        .iter()
        .sorted()
        .chunk_by(|date| date.date_naive())
        .into_iter()
        .map(|(year, chunks)| (year, chunks.count() as u32))
        .collect()
}

mod test {
    use std::ops::{Add, AddAssign};
    use std::sync::Mutex;

    use chrono::TimeDelta;

    use super::*;
    use crate::seismic::SeismicEvent;
    use crate::state::SeismicData;

    fn empty_state() -> AppState {
        Mutex::new(SeismicData::new())
    }

    fn state_with_one_entry() -> AppState {
        let state = empty_state();
        let mut guard = state.lock().unwrap();
        guard.add_or_update_event(SeismicEvent::test_event());
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
        for i in 0..size {
            let event = create(i);
            guard.add_or_update_event(event);
        }
        drop(guard);
        state
    }

    #[test]
    fn test_empty_count_by_year() {
        let state = empty_state();

        let response = get_count_by_year_internal(&state);

        assert_eq!(response.len(), 0);
    }

    #[test]
    fn test_single_count_by_year() {
        let state = state_with_one_entry();

        let response = get_count_by_year_internal(&state);

        assert_eq!(response.len(), 1);
        assert_eq!(
            response[0],
            (NaiveDate::from_ymd_opt(2024, 12, 10).unwrap(), 1)
        );
    }

    #[test]
    fn test_multiple_count_by_year() {
        let state = state_with_n_entries(3);

        let response = get_count_by_year_internal(&state);

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
            event
                .time
                .add_assign(TimeDelta::days(index.div_euclid(2) as i64));
            event
        });
        let start_date = NaiveDate::from_ymd_opt(2024, 12, 10).unwrap();

        let response = get_count_by_year_internal(&state);

        assert_eq!(response.len(), 3);
        assert_eq!(response[0], (start_date, 2));
        assert_eq!(response[2], (start_date.add(TimeDelta::days(2)), 1));
    }

    #[test]
    fn test_no_magnitudes() {
        let state = empty_state();

        let response = get_magnitude_distribution_internal(&state);

        assert_eq!(response.len(), 0);
    }

    #[test]
    fn test_one_magnitude() {
        let state = state_with_one_entry();

        let response = get_magnitude_distribution_internal(&state);

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].1, 1);
    }

    #[test]
    fn test_many_magnitudes() {
        let state = state_with_n_entries(4);

        let response = get_magnitude_distribution_internal(&state);

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

        let response = get_magnitude_distribution_internal(&state);

        assert_eq!(response.len(), 4);
        assert_eq!(response[0].1, 2);
        assert_eq!(response[2].0, "2.4");
        assert_eq!(response[3].1, 1);
    }
}
