pub mod commands;
pub mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .with_target(true)
        .init();

    tracing::info!("FerroTone starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::start_capture,
            commands::stop_capture,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
