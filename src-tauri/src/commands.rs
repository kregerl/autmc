use std::{collections::HashMap, hash::Hash, path::PathBuf};

use log::{debug, warn, error};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Wry};

use crate::{
    consts::{CLIENT_ID, MICROSOFT_LOGIN_URL},
    state::{
        account_manager::{AccountState},
        instance_manager::InstanceState,
        resource_manager::{ManifestResult, ResourceState}, redirect,
    },
    web_services::{
        authentication::{validate_account, AuthResult},
        manifest::vanilla::VanillaManifestVersion,
        resources::create_instance,
    },
};

#[cfg(target_family = "unix")]
fn get_init_script_for_os() -> String {
    r#"
        if (window.location.href.startsWith("https://login.microsoftonline.com/common/oauth2/nativeclient")) {
            window.location.replace(`autmc://auth${window.location.search}`);
        }
    "#.into()
}

#[cfg(target_family = "windows")]
fn get_init_script_for_os() -> String {
    r#"
        if (window.location.href.startsWith("https://login.microsoftonline.com/common/oauth2/nativeclient")) {
            window.location.replace(`https://autmc.auth${window.location.search}`);
        }
    "#.into()
}

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

    debug!("Init script injected");
    let init_script = get_init_script_for_os();
    // Redirects to the custom protocol 'autmc://auth', preserving the query parameters.
    let window_url = tauri::WindowUrl::App(login_url.to_string().parse().unwrap());
    // Start window with init script
    let _login_window = tauri::WindowBuilder::new(&app_handle, "login", window_url)
        .initialization_script(&init_script)
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
    forge_versions: HashMap<String, Vec<String>>,
}

#[tauri::command(async)]
pub async fn obtain_manifests(app_handle: AppHandle<Wry>) -> ManifestResult<VersionManifest> {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;

    let vanilla_versions = resource_manager.get_vanilla_version_list();
    let fabric_versions = resource_manager.get_fabric_version_list();
    let forge_versions = resource_manager.get_forge_version_list();

    Ok(VersionManifest {
        vanilla_versions,
        fabric_versions,
        forge_versions,
    })
}

#[tauri::command(async)]
pub async fn obtain_version(
    vanilla_version: String,
    modloader_type: String,
    modloader_version: String,
    instance_name: String,
    app_handle: AppHandle<Wry>,
) -> ManifestResult<Vec<String>> {
    debug!(
        "Creating instance {} from mc:{} ml:{} mlv:{}",
        instance_name, vanilla_version, modloader_type, modloader_version
    );
    create_instance(
        vanilla_version,
        modloader_type,
        modloader_version,
        instance_name,
        &app_handle,
    )
    .await?;
    let instance_state: State<InstanceState> = app_handle
        .try_state()
        .expect("`InstanceState` should already be managed.");
    let mut instance_manager = instance_state.0.lock().await;

    instance_manager.deserialize_instances();
    Ok(instance_manager.get_instance_names())
}

#[tauri::command(async)]
pub async fn get_instance_path(name: String, app_handle: AppHandle<Wry>) -> PathBuf {
    debug!("Name: {}", name);
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;

    resource_manager.instances_dir().join(name)
}

#[derive(Debug, Serialize)]
pub struct BasicAccount {
    uuid: String,
    name: String,
    skin_url: String,
}

#[derive(Debug, Serialize)]
pub struct AccountInformation {
    active_account: Option<String>,
    accounts: HashMap<String, BasicAccount>,
}

#[tauri::command(async)]
pub async fn get_accounts(app_handle: AppHandle<Wry>) -> AccountInformation {
    let account_state: tauri::State<AccountState> = app_handle
        .try_state()
        .expect("`AccountState` should already be managed.");
    let account_manager = account_state.0.lock().await;
    let uuid = account_manager.get_active_uuid();
    let accounts = account_manager
        .get_all_accounts()
        .into_iter()
        .map(|(key, value)| {
            (
                key,
                BasicAccount {
                    uuid: value.uuid,
                    name: value.name,
                    skin_url: value.skin_url,
                },
            )
        })
        .collect();
    AccountInformation {
        active_account: uuid,
        accounts,
    }
}

#[tauri::command(async)]
pub async fn login_to_account(uuid: String, app_handle: AppHandle<Wry>) {
    let account_state: tauri::State<AccountState> = app_handle
        .try_state()
        .expect("`AccountState` should already be managed.");
    let mut account_manager = account_state.0.lock().await;

    account_manager.activate_account(&uuid);

    // Get the active account that was just set.
    match account_manager.get_active_account() {
        Some(active_account) => {
            let validation_result = validate_account(active_account).await;

            // If the result if an error, emit error to user
            if let Err(validation_error) = &validation_result {
                if let Err(error) =
                    app_handle.emit_to("main", "authentication-error", validation_error.to_string())
                {
                    error!("{}", error.to_string());
                    return;
                }
            }

            if let Err(error) = account_manager.serialize_accounts() {
                warn!("Could not properly serialize account information: {}", error);
                return;
            }

            if let Err(error) = redirect(&app_handle, "") {
                error!("{}", error.to_string());
            }
        }
        None => {
            // FIXME: Emit error to user
            error!("No account with uuid: {}", uuid);
        }
    }
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
