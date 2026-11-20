//! Application layer of note-tice!: Tauri setup, commands, windows.

mod attachments;
mod commands;
mod data_dir;
mod state;

use anyhow::Context;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // Ignore the error if a global subscriber is already set (e.g. in tests).
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

/// Builds and runs the Tauri application.
pub fn run() -> anyhow::Result<()> {
    init_tracing();
    tracing::info!(version = notetice_core::VERSION, "starting note-tice!");
    let data_dir = data_dir::resolve()?;

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol(attachments::SCHEME, attachments::handle)
        .setup(move |app| {
            app.manage(state::AppState::open(&data_dir)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_workspaces,
            commands::create_workspace,
            commands::list_notes,
            commands::get_note,
            commands::create_note,
            commands::load_doc,
            commands::apply_local_update,
            commands::update_note_meta,
            commands::delete_note,
            commands::restore_note,
            commands::list_folders,
            commands::create_folder,
            commands::rename_folder,
            commands::delete_folder,
            commands::search,
            commands::get_settings,
            commands::set_setting,
            commands::save_attachment,
            commands::get_attachments,
            commands::write_export_files,
        ])
        .run(tauri::generate_context!())
        .context("failed to run the Tauri application")
}
