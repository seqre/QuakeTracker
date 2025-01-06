use itertools::Itertools;

use crate::AppState;

#[tauri::command]
pub fn get_magnitude_distribution(state: tauri::State<'_, AppState>) -> Vec<(String, u32)> {
    get_magnitude_distribution_internal(state.inner())
}

fn get_magnitude_distribution_internal(state: &AppState) -> Vec<(String, u32)> {
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

mod test {
    use std::sync::Mutex;

    use crate::analytics::get_magnitude_distribution_internal;
    use crate::seismic::SeismicEvent;
    use crate::state::SeismicData;

    #[test]
    fn test_no_magnitudes() {
        let state = Mutex::new(SeismicData::new());
        let response = get_magnitude_distribution_internal(&state);
        assert_eq!(response.len(), 0);
    }

    #[test]
    fn test_one_magnitude() {
        let state = Mutex::new(SeismicData::new());
        let mut guard = state.lock().unwrap();
        guard.add_or_update_event(SeismicEvent::test_event());
        drop(guard);

        let response = get_magnitude_distribution_internal(&state);
        assert_eq!(response.len(), 1);
        assert_eq!(response[0].1, 1);
    }

    #[test]
    fn test_many_magnitudes() {
        let state = Mutex::new(SeismicData::new());
        let mut guard = state.lock().unwrap();
        for i in 0..4 {
            let mut event = SeismicEvent::test_event();
            event.id.push_str(&i.to_string());
            guard.add_or_update_event(event);
        }
        drop(guard);

        let response = get_magnitude_distribution_internal(&state);
        assert_eq!(response.len(), 1);
        assert_eq!(response[0].1, 4);
    }

    #[test]
    fn test_many_magnitudes_bucketing() {
        let state = Mutex::new(SeismicData::new());
        let mut guard = state.lock().unwrap();
        for i in 0..7 {
            let mut event = SeismicEvent::test_event();
            event.id.push_str(&i.to_string());
            event.magnitude += 0.1 * (i as f64);
            guard.add_or_update_event(event);
        }
        drop(guard);

        let response = get_magnitude_distribution_internal(&state);
        assert_eq!(response.len(), 4);
        assert_eq!(response[0].1, 2);
        assert_eq!(response[2].0, "2.4");
        assert_eq!(response[3].1, 1);
    }
}
