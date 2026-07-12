use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::Manager;

use ferrotone_core::config::Settings;
use ferrotone_lib::state::AppState;

fn make_request(cmd: &str) -> InvokeRequest {
    InvokeRequest {
        cmd: cmd.into(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: "http://tauri.localhost".parse().unwrap(),
        body: InvokeBody::default(),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    }
}

#[test]
fn app_initializes_with_stop_command() {
    let app = mock_builder()
        .manage(AppState::new(Settings::default()))
        .invoke_handler(tauri::generate_handler![
            ferrotone_lib::commands::stop_capture,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build app");

    let _webview = app
        .get_webview_window("main")
        .expect("main webview should exist");
    let _state = app.state::<AppState>();
}

#[test]
fn stop_returns_success() {
    let app = mock_builder()
        .manage(AppState::new(Settings::default()))
        .invoke_handler(tauri::generate_handler![
            ferrotone_lib::commands::stop_capture,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build app");

    let webview = app.get_webview_window("main").expect("main webview");
    let result = get_ipc_response(&webview, make_request("stop_capture"));
    assert!(
        result.is_ok(),
        "stop should return success, got {:?}",
        result
    );
}

#[test]
fn unknown_command_returns_error() {
    let app = mock_builder()
        .manage(AppState::new(Settings::default()))
        .build(tauri::generate_context!())
        .expect("failed to build app");

    let webview = app.get_webview_window("main").expect("main webview");
    let result = get_ipc_response(&webview, make_request("nonexistent"));
    assert!(
        result.is_err(),
        "unknown command should error, got {:?}",
        result
    );
}
