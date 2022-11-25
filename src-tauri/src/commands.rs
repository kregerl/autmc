use std::time::Instant;

use log::info;
use reqwest::Url;
use tauri::{Wry, AppHandle, State, Manager};

use crate::{web_services::{
    authentication::AuthResult,
    consts::{CLIENT_ID, MICROSOFT_LOGIN_URL}, resources::{Library, JarType},
}, state::resource_manager::{ManifestResult, ResourceState, construct_arguments, rules_match}};

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
pub async fn obtain_version(selected: String, app_handle: AppHandle<Wry>) -> ManifestResult<()> {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;

    let start = Instant::now();

    let version = resource_manager.download_vanilla_version(&selected).await?;

    let libraries: Vec<Library> = version
        .libraries
        .into_iter()
        .filter_map(|lib| {
            // If we have any rules...
            if let Some(rules) = &lib.rules {
                // and the rules dont match
                if !rules_match(&rules) {
                    // remove
                    None
                } else {
                    // Otherwise keep lib in download list
                    Some(lib)
                }
            } else {
                // Otherwise keep lib in download list
                Some(lib)
            }
        })
        .collect();

    let lib_paths = resource_manager.download_libraries(&libraries).await?;

    let game_jar_path = resource_manager
        .download_game_jar(JarType::Client, &version.downloads.client, &version.id)
        .await?;

    let java_path = resource_manager
        .download_java_version(
            &version.java_version.component,
            version.java_version.major_version,
        )
        .await?;

    resource_manager
        .download_logging_configurations(&version.logging.client.file)
        .await?;

    resource_manager
        .download_assets(&version.asset_index)
        .await?;
    info!(
        "Finished download instance in {}ms",
        start.elapsed().as_millis()
    );

    // https://stackoverflow.com/questions/62186871/how-to-correctly-use-peek-in-rust
    construct_arguments(&version.arguments, &lib_paths, &game_jar_path);
    Ok(())
}