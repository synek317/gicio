use crate::launch::{Launch, LaunchArgs};
use tauri::State;

#[tauri::command]
pub fn launch_args(launch: State<'_, Launch>) -> Option<LaunchArgs> {
    launch.0.lock().unwrap().take()
}
