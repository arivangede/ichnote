//! Application layer of Ichnote: Tauri setup, commands, windows.

use anyhow::Context;
use tracing_subscriber::EnvFilter;

/// Returns the version of the app and its core library.
#[tauri::command]
fn app_version() -> String {
    ichnote_core::VERSION.to_owned()
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // Ignore the error if a global subscriber is already set (e.g. in tests).
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

/// Builds and runs the Tauri application.
pub fn run() -> anyhow::Result<()> {
    init_tracing();
    tracing::info!(version = ichnote_core::VERSION, "starting Ichnote");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_version])
        .run(tauri::generate_context!())
        .context("failed to run the Tauri application")
}
