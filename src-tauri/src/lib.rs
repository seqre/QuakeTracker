mod analytics;
mod client;
mod commands;
mod seismic;
mod state;

#[cfg(test)]
pub(crate) mod test_utils;

use std::error::Error;
use std::sync::Mutex;

use state::SeismicData;
use tauri::{App, Manager, Runtime};
pub type AppState = Mutex<SeismicData>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_seismic_events,
            commands::listen_to_seismic_events,
            commands::get_magnitude_distribution,
            commands::get_count_by_year,
            commands::get_mag_depth_pairs,
            commands::get_advanced_analytics,
            commands::get_data_stats,
            commands::recompute_analytics,
            commands::get_hourly_frequency,
            commands::get_monthly_frequency,
            commands::get_weekly_frequency,
            commands::get_region_hotspots,
            commands::get_coordinate_clusters,
            commands::get_b_value,
            commands::get_magnitude_frequency_data,
            commands::get_risk_metrics,
            commands::get_total_energy,
        ])
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup<R: Runtime>(app: &mut App<R>) -> Result<(), Box<dyn Error>> {
    // #[cfg(debug_assertions)] // only include this code on debug builds
    // {
    //     let window = app.get_webview_window("main").unwrap();
    //     window.open_devtools();
    // }

    app.manage(Mutex::new(SeismicData::default()));
    Ok(())
}
