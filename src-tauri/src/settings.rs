//! App-wide settings (settings window), stored as JSON in the app config directory. The
//! backend only interprets the AI instructions; everything else belongs to the UI.

use gicio_core::ai::ExtraInstructions;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub ai_instructions: ExtraInstructions,
    #[serde(flatten)]
    pub ui: Map<String, Value>,
}

#[derive(Default)]
pub struct SettingsStore {
    current: Mutex<AppSettings>,
}

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("settings.json"))
}

impl SettingsStore {
    pub fn load(&self, app: &AppHandle) {
        let loaded = path(app).and_then(|p| std::fs::read(p).ok()).and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        *self.current.lock().unwrap() = loaded;
    }

    pub fn get(&self) -> AppSettings {
        self.current.lock().unwrap().clone()
    }

    pub fn save(&self, app: &AppHandle, s: AppSettings) -> Result<(), String> {
        let p = path(app).ok_or("no config directory")?;
        if let Some(d) = p.parent() {
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        std::fs::write(&p, serde_json::to_vec_pretty(&s).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        *self.current.lock().unwrap() = s;
        Ok(())
    }

    pub fn instructions(&self) -> ExtraInstructions {
        self.current.lock().unwrap().ai_instructions.clone()
    }
}
