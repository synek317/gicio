mod commands;
mod jobs;
mod publish;
mod settings;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            use tauri::Manager;
            app.state::<AppState>().settings.load(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session::open_session,
            commands::session::get_session,
            commands::session::refresh,
            commands::session::list_refs,
            commands::session::file_texts,
            commands::session::read_file,
            commands::session::reset_review,
            commands::session::save_file,
            commands::marks::set_marks,
            commands::marks::undo,
            commands::plan::plan_ai,
            commands::plan::replan_ai,
            commands::threads::create_thread,
            commands::threads::send_message,
            commands::threads::set_thread_status,
            commands::review::submit_review,
            commands::review::export_review,
            commands::review::ai_review,
            commands::review::set_findings_status,
            commands::review::fix_findings,
            commands::git_ops::staged_stat,
            commands::git_ops::suggest_commit_message,
            commands::git_ops::commit,
            commands::tools::run_tool,
            commands::session::set_diff_options,
            commands::session::write_text_file,
            commands::lsp::lsp_symbols,
            commands::chat::chat_send,
            commands::chat::summarize,
            commands::chat::clear_chat,
            commands::settings::get_app_settings,
            commands::settings::save_app_settings,
            commands::jobs::list_jobs,
            commands::jobs::cancel_job,
            commands::jobs::set_backend,
            commands::lsp::lsp_status,
            commands::lsp::lsp_hover,
            commands::lsp::lsp_definition,
            commands::lsp::lsp_references,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
