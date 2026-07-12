# tauri 2.0.0 test module API Summary

Available with feature flag `test`.

## Functions
- `mock_builder() -> Builder<MockRuntime>` — creates Builder with MockRuntime
- `mock_context() -> Context<NoopAsset>` — creates test Context
- `mock_app() -> App<MockRuntime>` — creates App with mock context and noop assets
- `noop_assets() -> NoopAsset` — empty Assets impl
- `get_ipc_response<R: DeserializeOwned>(webview: &WebviewWindow<MockRuntime>, request: InvokeRequest) -> Result<R>` — executes IPC and gets response
- `assert_ipc_response<R: DeserializeOwned + PartialEq + Debug>(webview: &WebviewWindow<MockRuntime>, request: InvokeRequest, expected: R)` — asserts IPC response

## Types
- `MockRuntime`, `MockRuntimeHandle`, `MockWebviewDispatcher`, `MockWindowBuilder`, `MockWindowDispatcher`
- `NoopAsset`
- `RuntimeContext`
- `EventProxy`

## Constants
- `INVOKE_KEY: &str` — invoke key for tests

## Pattern
```rust
use tauri::test::{mock_builder, mock_context, noop_assets, get_ipc_response, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::ipc::{CallbackFn, InvokeBody};

let app = mock_builder().build(tauri::generate_context!()).unwrap();
let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();

let res: String = get_ipc_response(&webview, InvokeRequest {
    cmd: "ping".into(),
    callback: CallbackFn(0),
    error: CallbackFn(1),
    url: "http://tauri.localhost".parse().unwrap(),
    body: InvokeBody::default(),
    headers: Default::default(),
    invoke_key: INVOKE_KEY.to_string(),
}).unwrap();
```
