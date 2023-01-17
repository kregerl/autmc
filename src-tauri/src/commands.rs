use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Child,
};

use log::debug;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Wry};

use crate::{
    consts::{CLIENT_ID, MICROSOFT_LOGIN_URL},
    state::{
        account_manager::AccountState,
        instance_manager::InstanceState,
        resource_manager::{ManifestResult, ResourceState},
    },
    web_services::{
        authentication::AuthResult, manifest::vanilla::VanillaManifestVersion,
        resources::create_instance,
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

#[derive(Serialize)]
pub struct VersionManifest {
    vanilla_versions: Vec<VersionEntry>,
    fabric_versions: Vec<String>,
}

#[tauri::command(async)]
pub async fn obtain_manifests(
    filters: Vec<VersionFilter>,
    app_handle: AppHandle<Wry>,
) -> ManifestResult<VersionManifest> {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;

    let vanilla_versions = resource_manager.get_vanilla_version_list(&filters);
    let fabric_versions = resource_manager.get_fabric_version_list();
    
    Ok(VersionManifest {
        vanilla_versions,
        fabric_versions
    })
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

    // FIXME: Unwraping here causes an error sometimes since the async thread (in main) isnt finished yet and there is no active account loaded.
    let account = account_manager.get_active_account().unwrap();
    debug!("Skin URL: {}", account.skin_url);
    account.skin_url.clone()
}

#[tauri::command(async)]
pub async fn load_instances(app_handle: AppHandle<Wry>) -> Vec<String> {
    let instance_state: State<InstanceState> = app_handle
        .try_state()
        .expect("`InstanceState` should already be managed.");
    let mut instance_manager = instance_state.0.lock().await;

    instance_manager.deserialize_instances();
    instance_manager.get_instance_names()
}

#[tauri::command(async)]
pub async fn launch_instance(instance_name: String, app_handle: AppHandle<Wry>) {
    let instance_state: State<InstanceState> = app_handle
        .try_state()
        .expect("`InstanceState` should already be managed.");
    let mut instance_manager = instance_state.0.lock().await;

    let account_state: State<AccountState> = app_handle
        .try_state()
        .expect("`AccountState` should already be managed.");

    let account_manager = account_state.0.lock().await;

    // Assumed there is an active account.
    instance_manager.launch_instance(
        &instance_name,
        account_manager.get_active_account().unwrap(),
    );
    instance_manager.emit_logs_for_running_instance(app_handle.clone());
}
