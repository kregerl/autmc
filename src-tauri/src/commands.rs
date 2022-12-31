use std::path::PathBuf;

use log::debug;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Wry};

use crate::{
    consts::{CLIENT_ID, MICROSOFT_LOGIN_URL},
    state::{resource_manager::{ManifestResult, ResourceState}, account_manager::AccountState},
    web_services::{
        authentication::AuthResult,
        resources::{create_instance}, manifest::vanilla::VanillaManifestVersion,
    },
};

#[tauri::command(async)]
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

#[derive(Deserialize)]
pub struct VersionFilter {
    pub id: String,
    pub name: String,
    pub checked: bool,
}

#[derive(Serialize)]
pub struct VersionEntry {
    version: String,
    #[serde(rename = "releasedDate")]
    released_date: String,
    #[serde(rename = "versionType")]
    version_type: String,
}

impl VersionEntry {
    pub fn new(version: &str, version_info: &VanillaManifestVersion) -> Self {
        Self {
            version: version.into(),
            released_date: version_info.release_time.clone(),
            version_type: version_info.version_type.clone(),
        }
    }
}

#[tauri::command(async)]
pub async fn obtain_manifests(
    filters: Vec<VersionFilter>,
    app_handle: AppHandle<Wry>,
) -> ManifestResult<Vec<VersionEntry>> {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;

    let versions = resource_manager.get_vanilla_version_list(&filters);
    Ok(versions)
}

#[tauri::command(async)]
pub async fn obtain_version(
    selected: String,
    instance_name: String,
    app_handle: AppHandle<Wry>,
) -> ManifestResult<()> {
    create_instance(selected, instance_name, &app_handle).await?;
    Ok(())
}

#[tauri::command(async)]
pub async fn get_instance_path(app_handle: AppHandle<Wry>) -> PathBuf {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;

    resource_manager.instances_dir()
}

#[tauri::command(async)]
pub async fn get_account_skin(app_handle: AppHandle<Wry>) -> String {
    let account_state: State<AccountState> = app_handle
        .try_state()
        .expect("`AccountState` should already be managed.");
    let account_manager = account_state.0.lock().await;

    // FIXME: Unwraping. 
    let account = account_manager.get_active_account().unwrap();
    debug!("Skin URL: {}", account.skin_url);
    account.skin_url.clone()
}

#[tauri::command(async)]
pub async fn load_instances(app_handle: AppHandle<Wry>) -> Vec<String> {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let mut resource_manager = resource_state.0.lock().await;

    resource_manager.deserialize_instances();
    resource_manager.get_instance_names()
}

#[tauri::command(async)]
pub async fn launch_instance(instance_name: String, app_handle: AppHandle<Wry>) {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;

    let account_state: State<AccountState> = app_handle
        .try_state()
        .expect("`AccountState` should already be managed.");

    let account_manager = account_state.0.lock().await;

    // Assumed there is an active account. 
    resource_manager.launch_instance(&instance_name, account_manager.get_active_account().unwrap());
}