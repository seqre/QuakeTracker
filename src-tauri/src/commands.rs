use chrono::NaiveDate;
use futures_util::StreamExt;
use tauri::ipc::Channel;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

use crate::client::{Error, QueryParams, WssEvent, SEISMIC_WSS_URL};
use crate::{analytics, client, AppState};

#[tauri::command]
pub fn get_magnitude_distribution(state: tauri::State<'_, AppState>) -> Vec<(String, u32)> {
    analytics::get_magnitude_distribution_internal(state.inner())
}

#[tauri::command]
pub fn get_count_by_year(state: tauri::State<'_, AppState>) -> Vec<(NaiveDate, u32)> {
    analytics::get_count_by_year_internal(state.inner())
}
#[tauri::command]
pub fn get_mag_depth_pairs(state: tauri::State<'_, AppState>) -> Vec<(f64, f64)> {
    analytics::get_mag_depth_pairs_internal(state.inner())
}

#[tauri::command]
pub async fn get_seismic_events(
    state: tauri::State<'_, AppState>,
    query_params: QueryParams,
    clear: bool,
) -> Result<tauri::ipc::Response, Error> {
    if clear {
        let mut state = state.lock().unwrap();
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
) -> Result<(), Error> {
    let request = SEISMIC_WSS_URL.into_client_request().unwrap();

    let (mut stream, _response) = connect_async(request).await.unwrap();

    while let Some(msg) = stream.next().await {
        if let Ok(Message::Text(text)) = msg {
            let wss_event: WssEvent = serde_json::from_str(text.as_str()).unwrap();
            log::trace!("WSS Message: {wss_event:?}");

            let mut state = state.lock().unwrap();
            state.add_or_update_event(wss_event.data.clone());

            if let Err(e) = on_event.send(wss_event) {
                log::error!("{}", Error::Ipc(e.to_string()));
            }
        }
    }

    Ok(())
}
