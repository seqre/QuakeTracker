use chrono::NaiveDate;
use futures_util::StreamExt;
use tauri::ipc::Channel;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

use crate::client::{ClientResult, QueryParams, WssEvent, SEISMIC_WSS_URL};
use crate::{analytics, client, AppState};

#[tauri::command]
pub fn get_magnitude_distribution(state: tauri::State<'_, AppState>) -> Result<Vec<(String, u32)>, String> {
    analytics::get_magnitude_distribution_internal(state.inner())
}

#[tauri::command]
pub fn get_count_by_year(state: tauri::State<'_, AppState>) -> Result<Vec<(NaiveDate, u32)>, String> {
    analytics::get_count_by_year_internal(state.inner())
}

#[tauri::command]
pub fn get_mag_depth_pairs(state: tauri::State<'_, AppState>) -> Result<Vec<(f64, f64)>, String> {
    analytics::get_mag_depth_pairs_internal(state.inner())
}

#[tauri::command]
pub fn get_advanced_analytics(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    analytics::get_advanced_analytics_internal(state.inner())
}

#[tauri::command]
pub fn get_data_stats(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;
    let stats = state.get_stats();

    serde_json::to_value(stats).map_err(|e| format!("Failed to serialize stats: {}", e))
}

#[tauri::command]
pub async fn get_seismic_events(
    state: tauri::State<'_, AppState>,
    query_params: QueryParams,
    clear: bool,
) -> ClientResult<tauri::ipc::Response> {
    if clear {
        let mut state = state.lock()
            .map_err(|e| crate::client::ClientError::Internal(format!("Failed to acquire state lock: {}", e)))?;
        state.clear();
    }
    let events = client::get_seismic_events_internal(state.inner(), query_params).await?;
    Ok(tauri::ipc::Response::new(events))
}

// https://www.seismicportal.eu/realtime.html
#[tauri::command]
pub async fn listen_to_seismic_events(
    state: tauri::State<'_, AppState>,
    on_event: Channel<WssEvent>,
) -> ClientResult<()> {
    log::info!("Starting WebSocket connection to EMSC with retry logic");
    
    const MAX_RETRIES: u32 = 5;
    const INITIAL_DELAY_MS: u64 = 1000;
    
    let mut retry_count = 0;
    let mut delay = INITIAL_DELAY_MS;
    
    while retry_count < MAX_RETRIES {
        match connect_and_listen(&state, &on_event).await {
            Ok(_) => {
                log::info!("WebSocket connection closed gracefully");
                return Ok(());
            }
            Err(e) => {
                retry_count += 1;
                log::error!("WebSocket connection failed (attempt {}/{}): {}", retry_count, MAX_RETRIES, e);
                
                if retry_count >= MAX_RETRIES {
                    log::error!("Max retry attempts reached, giving up");
                    return Err(e);
                }
                
                log::info!("Retrying in {}ms...", delay);
                sleep(Duration::from_millis(delay)).await;
                
                // Exponential backoff with cap at 30 seconds
                delay = std::cmp::min(delay * 2, 30000);
            }
        }
    }
    
    Err(crate::client::ClientError::Network("Failed to connect after all retries".to_string()))
}

async fn connect_and_listen(
    state: &tauri::State<'_, AppState>,
    on_event: &Channel<WssEvent>,
) -> ClientResult<()> {
    let request = SEISMIC_WSS_URL.into_client_request()
        .map_err(|e| crate::client::ClientError::Network(format!("Invalid WebSocket URL: {}", e)))?;

    let (mut stream, _response) = connect_async(request).await
        .map_err(|e| crate::client::ClientError::Network(format!("WebSocket connection failed: {}", e)))?;

    log::info!("WebSocket connected successfully");

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match handle_websocket_message(&text, state, on_event).await {
                    Ok(_) => {},
                    Err(e) => {
                        log::error!("Error handling WebSocket message: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                log::info!("WebSocket closed by server");
                break;
            }
            Ok(_) => {
                log::warn!("Received unexpected message");
            }
            Err(e) => {
                log::error!("WebSocket error: {}", e);
                return Err(crate::client::ClientError::Network(format!("WebSocket error: {}", e)));
            }
        }
    }

    Ok(())
}

async fn handle_websocket_message(
    text: &str,
    state: &tauri::State<'_, AppState>,
    on_event: &Channel<WssEvent>,
) -> ClientResult<()> {
    log::trace!("Received WebSocket message: {}", text);

    let wss_event: WssEvent = serde_json::from_str(text)
        .map_err(|e| crate::client::ClientError::Parse(format!("Failed to parse WebSocket message: {}", e)))?;

    log::debug!("Parsed WebSocket event: {:?}", wss_event);

    // Add event to state
    {
        let mut state_guard = state.lock()
            .map_err(|e| crate::client::ClientError::Internal(format!("Failed to acquire state lock: {}", e)))?;
        
        state_guard.add_or_update_event(wss_event.data.clone())
            .map_err(|e| crate::client::ClientError::Internal(format!("Failed to add event to state: {}", e)))?;
    }

    // Send event to frontend
    if let Err(e) = on_event.send(wss_event) {
        log::error!("Failed to send event to frontend: {}", e);
        return Err(crate::client::ClientError::Internal(format!("Failed to send event to frontend: {}", e)));
    }

    Ok(())
}

#[tauri::command]
pub fn recompute_analytics(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let state = state
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;
    state
        .recompute_analytics()
        .map_err(|e| format!("Failed to recompute analytics: {}", e))
}

#[tauri::command]
pub fn get_hourly_frequency(state: tauri::State<'_, AppState>) -> Result<Vec<(u32, u32)>, String> {
    analytics::get_hourly_frequency_internal(state.inner())
}

#[tauri::command]
pub fn get_monthly_frequency(state: tauri::State<'_, AppState>) -> Result<Vec<(u32, u32)>, String> {
    analytics::get_monthly_frequency_internal(state.inner())
}

#[tauri::command]
pub fn get_weekly_frequency(state: tauri::State<'_, AppState>) -> Result<Vec<(String, u32)>, String> {
    analytics::get_weekly_frequency_internal(state.inner())
}

#[tauri::command]
pub fn get_region_hotspots(state: tauri::State<'_, AppState>) -> Result<Vec<(String, u32)>, String> {
    analytics::get_region_hotspots_internal(state.inner())
}

#[tauri::command]
pub fn get_coordinate_clusters(state: tauri::State<'_, AppState>) -> Result<Vec<(f64, f64, u32)>, String> {
    analytics::get_coordinate_clusters_internal(state.inner())
}

#[tauri::command]
pub fn get_b_value(state: tauri::State<'_, AppState>) -> Result<f64, String> {
    analytics::get_b_value_internal(state.inner())
}

#[tauri::command]
pub fn get_magnitude_frequency_data(state: tauri::State<'_, AppState>) -> Result<Vec<(f64, u32, u32)>, String> {
    analytics::get_magnitude_frequency_data_internal(state.inner())
}

#[tauri::command]
pub fn get_risk_metrics(state: tauri::State<'_, AppState>) -> Result<(f64, f64, f64, f64), String> {
    analytics::get_risk_metrics_internal(state.inner())
}

#[tauri::command]
pub fn get_total_energy(state: tauri::State<'_, AppState>) -> Result<f64, String> {
    analytics::get_total_energy_internal(state.inner())
}
