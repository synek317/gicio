use crate::settings::AppSettings;
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_app_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.get()
}

#[tauri::command]
pub fn save_app_settings(app: AppHandle, state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    state.settings.save(&app, settings)
}
