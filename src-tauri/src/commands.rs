use reqwest::Url;
use tauri::{AppHandle, Manager, State, Wry};

use crate::{
    consts::{CLIENT_ID, MICROSOFT_LOGIN_URL},
    state::resource_manager::{ManifestResult, ResourceState},
    web_services::{authentication::AuthResult, resources::create_instance},
};

#[tauri::command]
pub async fn show_microsoft_login_page(app_handle: tauri::AppHandle<Wry>) -> AuthResult<()> {
    let login_url = Url::parse_with_params(
        MICROSOFT_LOGIN_URL,
        &[
            ("prompt", "select_account"),
            ("client_id", CLIENT_ID),
            ("response_type", "code"),
            ("scope", "XboxLive.signin offline_access"),
            (
                "redirect_uri",
                // TODO: Replace with REDIRECT_URL?
                "https://login.microsoftonline.com/common/oauth2/nativeclient",
            ),
        ],
    )?;

    // Redirects to the custom protocol 'autmc://auth', preserving the query parameters.
    const INIT_SCRIPT: &str = r#"
        if (window.location.href.startsWith("https://login.microsoftonline.com/common/oauth2/nativeclient")) {
            window.location.replace(`autmc://auth${window.location.search}`);
        }
    "#;
    let window_url = tauri::WindowUrl::App(login_url.to_string().parse().unwrap());
    // Start window with init script
    let _login_window = tauri::WindowBuilder::new(&app_handle, "login", window_url)
        .initialization_script(INIT_SCRIPT)
        .build()?;
    Ok(())
}

#[tauri::command]
pub async fn obtain_manifests(
    show_snapshots: bool,
    app_handle: AppHandle<Wry>,
) -> ManifestResult<Vec<String>> {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;

    let versions = resource_manager.get_vanilla_version_list(show_snapshots);
    Ok(versions)
}

#[tauri::command]
pub async fn obtain_version(selected: String, instance_name: String, app_handle: AppHandle<Wry>) -> ManifestResult<()> {
    create_instance(selected, instance_name, &app_handle).await?;
    Ok(())
}
