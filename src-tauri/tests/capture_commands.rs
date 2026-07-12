use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::Manager;

use ferrotone_core::config::Settings;
use ferrotone_lib::state::AppState;

fn build_app() -> tauri::App<tauri::test::MockRuntime> {
    mock_builder()
        .manage(AppState::new(Settings::default()))
        .invoke_handler(tauri::generate_handler![
            ferrotone_lib::commands::stop_capture,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build app")
}

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
fn stop_without_start_returns_ok() {
    let app = build_app();
    let webview = app.get_webview_window("main").expect("main webview");
    let result = get_ipc_response(&webview, make_request("stop_capture"));
    assert!(
        result.is_ok(),
        "stopping without start should be ok, got {:?}",
        result
    );
}

#[test]
fn repeated_stop_is_safe() {
    let app = build_app();
    let webview = app.get_webview_window("main").expect("main webview");
    for _ in 0..5 {
        let result = get_ipc_response(&webview, make_request("stop_capture"));
        assert!(result.is_ok(), "stop should succeed, got {:?}", result);
    }
}

#[test]
fn state_is_initialized_none() {
    let app = build_app();
    let state = app.state::<AppState>();
    let engine = state.engine.lock().unwrap();
    assert!(engine.is_none(), "engine should be None initially");
}
