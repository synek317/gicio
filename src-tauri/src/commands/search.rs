use crate::publish::err;
use crate::state::AppState;
use gicio_core::search::{self, SearchQuery, SearchResults};
use tauri::State;

/// Searches the files of the repository under review (off the main thread: it reads every file).
#[tauri::command]
pub async fn search_project(state: State<'_, AppState>, query: SearchQuery) -> Result<SearchResults, String> {
    let root = state.session()?.state.repo.clone();
    tauri::async_runtime::spawn_blocking(move || search::search_project(&root, &query)).await.map_err(err)?.map_err(err)
}
