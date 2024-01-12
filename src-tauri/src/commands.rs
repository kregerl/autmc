use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use autmc_authentication::{
    poll_device_code_status, start_device_code_authentication, AuthenticationResult, DeviceCode,
};
use flate2::read::GzDecoder;
use log::{debug, error, info, warn};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Wry};
use zip::ZipArchive;

use crate::{
    consts::{CLIENT_ID, GZIP_SIGNATURE, MICROSOFT_LOGIN_URL},
    state::{
        account_manager::AccountState,
        instance_manager::{InstanceConfiguration, InstanceState},
        resource_manager::{ManifestResult, ResourceState},
    },
    web_services::{
        manifest::{path_to_utf8_str, vanilla::VanillaManifestVersion},
        modpack::{
            curseforge::{
                import_curseforge_zip, retrieve_curseforge_categories, search_curseforge_modpacks,
                CurseforgeCategory, CurseforgeSearchAuthors, CurseforgeSearchEntry,
                CurseforgeSearchImage, CurseforgeSortField,
            },
            modrinth::import_modrinth_zip,
        },
        resources::{create_instance, InstanceSettings},
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
pub async fn start_authentication_flow() -> AuthenticationResult<DeviceCode> {
    let device_code = start_device_code_authentication().await?;
    debug!("Got device code: {:#?}", device_code);
    Ok(device_code)
}

#[tauri::command(async)]
pub async fn poll_device_code_authentication(
    device_code: String,
    app_handle: tauri::AppHandle<Wry>,
) -> AuthenticationResult<()> {
    let account = poll_device_code_status(&device_code).await?;
    debug!("Got Account: {:#?}", account);

    let account_state: tauri::State<AccountState> = app_handle
        .try_state()
        .expect("`AccountState` should already be managed.");
    let mut account_manager = account_state.0.lock().await;

    // Save account to account manager.
    account_manager.add_and_activate_account(account, app_handle.clone());

    if let Err(error) = account_manager.serialize_accounts() {
        warn!(
            "Could not properly serialize account information: {}",
            error
        );
    }
    Ok(())
}

// #[tauri::command(async)]
// pub async fn show_microsoft_login_page(app_handle: tauri::AppHandle<Wry>) -> Au<()> {
//     let login_url = Url::parse_with_params(
//         MICROSOFT_LOGIN_URL,
//         &[
//             ("prompt", "select_account"),
//             ("client_id", CLIENT_ID),
//             ("response_type", "code"),
//             ("scope", "XboxLive.signin offline_access"),
//             (
//                 "redirect_uri",
//                 "https://login.microsoftonline.com/common/oauth2/nativeclient",
//             ),
//         ],
//     )?;

//     debug!("Init script injected");
//     let init_script = get_init_script_for_os();
//     // Redirects to the custom protocol 'autmc://auth', preserving the query parameters.
//     let window_url = tauri::WindowUrl::App(login_url.to_string().parse().unwrap());
//     // Start window with init script
//     let _login_window = tauri::WindowBuilder::new(&app_handle, "login", window_url)
//         .initialization_script(&init_script)
//         .build()?;
//     Ok(())
// }

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
    let mut resource_manager = resource_state.0.lock().await;

    let vanilla_versions = resource_manager.get_vanilla_version_list().await?;
    let fabric_versions = resource_manager.get_fabric_version_list().await?;
    let forge_versions = resource_manager.get_forge_version_list().await?;

    Ok(VersionManifest {
        vanilla_versions,
        fabric_versions,
        forge_versions,
    })
}

#[tauri::command(async)]
pub async fn obtain_version(
    settings: InstanceSettings,
    app_handle: AppHandle<Wry>,
) -> ManifestResult<()> {
    debug!("Settings: {:#?}", settings);
    info!(
        "Creating instance {} with Minecraft version {} and modloader {} {}",
        settings.instance_name,
        settings.vanilla_version,
        settings.modloader_type.to_string(),
        settings.modloader_version
    );
    let instance_name = settings.instance_name.clone();

    create_instance(settings, &app_handle, None).await?;
    let instance_state: State<InstanceState> = app_handle
        .try_state()
        .expect("`InstanceState` should already be managed.");
    let mut instance_manager = instance_state.0.lock().await;

    instance_manager.deserialize_instances();
    app_handle.emit_all("new-instance", instance_name).unwrap();
    Ok(())
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

// #[tauri::command(async)]
// pub async fn login_to_account(uuid: String, app_handle: AppHandle<Wry>) {
//     let account_state: tauri::State<AccountState> = app_handle
//         .try_state()
//         .expect("`AccountState` should already be managed.");
//     let mut account_manager = account_state.0.lock().await;

//     account_manager.activate_account(&uuid, app_handle.clone());

//     // Get the active account that was just set.
//     match account_manager.get_active_account() {
//         Some(active_account) => {
//             let validation_result = validate_account(active_account).await;

//             // If the result if an error, emit error to user
//             if let Err(validation_error) = &validation_result {
//                 if let Err(error) =
//                     app_handle.emit_to("main", "authentication-error", validation_error.to_string())
//                 {
//                     error!("{}", error.to_string());
//                     return;
//                 }
//             }

//             if let Err(error) = account_manager.serialize_accounts() {
//                 warn!(
//                     "Could not properly serialize account information: {}",
//                     error
//                 );
//             }
//         }
//         None => {
//             // FIXME: Emit error to user
//             error!("No account with uuid: {}", uuid);
//         }
//     }
// }

#[tauri::command(async)]
pub async fn get_account_skin(app_handle: AppHandle<Wry>) -> String {
    let account_state: State<AccountState> = app_handle
        .try_state()
        .expect("`AccountState` should already be managed.");
    let account_manager = account_state.0.lock().await;

    let account = account_manager.get_active_account().unwrap();
    debug!("Skin URL: {}", account.skin_url);
    account.skin_url.clone()
}

#[tauri::command(async)]
pub async fn load_instances(app_handle: AppHandle<Wry>) -> Vec<InstanceConfiguration> {
    let instance_state: State<InstanceState> = app_handle
        .try_state()
        .expect("`InstanceState` should already be managed.");
    let mut instance_manager = instance_state.0.lock().await;

    instance_manager.deserialize_instances();
    debug!("load_instances");
    instance_manager.get_instance_configurations()
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
        app_handle.clone(),
    );
}

// FIXME: Instance names can be different from the directory name its stored in.
#[tauri::command(async)]
pub async fn open_folder(instance_name: String, app_handle: AppHandle<Wry>) {
    debug!("open_folder with name: {}", instance_name);
    let instance_state: State<InstanceState> = app_handle
        .try_state()
        .expect("`InstanceState` should already be managed.");
    let instance_manager = instance_state.0.lock().await;

    // Determine the command to open the default file explorer
    let command = match env::consts::OS {
        "linux" => "xdg-open",
        "macos" => "open",
        "windows" => "explorer",
        _ => unimplemented!(
            "Cannot open file explorer, unknown OS type: {}",
            env::consts::OS
        ),
    };

    // Spawn process of file explorer, can outlive parent.
    let result = Command::new(command)
        .arg(instance_manager.instances_dir().join(instance_name))
        .stdout(Stdio::null())
        .spawn();

    if let Err(e) = result {
        error!("Error spawning file manager process: {}", e);
    }
}

#[tauri::command(async)]
pub async fn get_screenshots(app_handle: AppHandle<Wry>) -> HashMap<String, Vec<String>> {
    let instance_state: State<InstanceState> = app_handle
        .try_state()
        .expect("`InstanceState` should already be managed.");
    let instance_manager = instance_state.0.lock().await;
    let instance_dir = instance_manager.instances_dir();

    let mut instance_screenshots = HashMap::new();
    for instance in instance_manager.get_instance_names() {
        let paths = fs::read_dir(instance_dir.join(&instance).join("screenshots"));

        if let Ok(paths) = paths {
            let mut screenshots: Vec<String> = Vec::new();
            for path in paths {
                let file_name = path.unwrap().file_name();
                let file_name_str = file_name.to_str().unwrap();
                let path = app_handle
                    .path_resolver()
                    .app_config_dir()
                    .unwrap()
                    .join(format!(
                        "instances/{}/screenshots/{}",
                        &instance, file_name_str
                    ));
                screenshots.push(path_to_utf8_str(&path).into());
            }
            instance_screenshots.insert(instance, screenshots);
        }
    }
    info!(
        "Found {} screenshots across all intances",
        instance_screenshots.len()
    );
    instance_screenshots
}

fn create_instance_log_map(
    instance_dir: &Path,
    instance_names: &[String],
) -> io::Result<HashMap<String, Vec<String>>> {
    let mut result = HashMap::new();

    for instance in instance_names {
        let directory_entries = fs::read_dir(instance_dir.join(instance).join("logs"));
        if directory_entries.is_err() {
            result.insert(instance.clone(), Vec::new());
            continue;
        }

        // Traverse every entry in the dir
        for dir_entry in directory_entries.unwrap() {
            let path = dir_entry?.path();
            if path.is_file() {
                let filename = path.file_name().unwrap().to_str().unwrap().into();
                if result.contains_key(instance) {
                    let existing_vec = result.get_mut(instance).unwrap();
                    existing_vec.push(filename);
                } else {
                    result.insert(instance.to_owned(), vec![filename]);
                }
            }
        }
    }

    Ok(result)
}

#[tauri::command(async)]
pub async fn get_logs(app_handle: AppHandle<Wry>) -> HashMap<String, Vec<String>> {
    let instance_state: State<InstanceState> = app_handle
        .try_state()
        .expect("`InstanceState` should already be managed.");
    let instance_manager = instance_state.0.lock().await;
    let instance_dir = instance_manager.instances_dir();

    match create_instance_log_map(&instance_dir, &instance_manager.get_instance_names()) {
        Ok(map) => map,
        Err(e) => {
            error!("Error creating logging maps: {}", e);
            HashMap::new()
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
enum LineType {
    Unknown,
    Normal,
    Error,
    Warning,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaggedLine {
    line: String,
    line_type: LineType,
}

fn get_tag_for_line(line: &String) -> LineType {
    if line.contains("/ERROR]:") {
        LineType::Error
    } else if line.contains("/WARN]:") {
        LineType::Warning
    } else if line.contains("/INFO]:") || line.contains("/DEBUG]:") {
        LineType::Normal
    } else {
        LineType::Unknown
    }
}

// Read bytes of log file and extract lines, decompressing gzip'd files if necessary
pub fn read_log_file(path: &Path) -> io::Result<Vec<TaggedLine>> {
    let bytes = fs::read(path)?;
    let lines: Vec<String> = if !bytes.is_empty() && bytes[..2] == GZIP_SIGNATURE {
        let mut decoder = GzDecoder::new(bytes.as_slice());
        let mut tmp_str = String::new();
        decoder.read_to_string(&mut tmp_str)?;

        tmp_str.lines().map(|line| line.into()).collect()
    } else {
        BufReader::new(bytes.as_slice())
            .lines()
            .filter_map(|line| line.ok())
            .collect()
    };
    let mut tagged_lines = Vec::with_capacity(lines.len());
    let mut previous_tag = LineType::Normal;
    for line in lines.into_iter() {
        let line_type = get_tag_for_line(&line);
        tagged_lines.push(if line_type != LineType::Unknown {
            previous_tag = line_type.clone();
            TaggedLine { line, line_type }
        } else {
            TaggedLine {
                line,
                line_type: previous_tag.clone(),
            }
        });
    }
    debug!("Done tagging log lines");

    Ok(tagged_lines)
}

#[tauri::command(async)]
pub async fn read_log_lines(
    instance_name: String,
    log_name: String,
    app_handle: AppHandle<Wry>,
) -> Vec<TaggedLine> {
    info!("Getting logs for {}", log_name);
    let instance_state: State<InstanceState> = app_handle
        .try_state()
        .expect("`InstanceState` should already be managed.");
    let instance_manager = instance_state.0.lock().await;
    let instance_dir = instance_manager.instances_dir();

    let path = instance_dir.join(instance_name).join("logs").join(log_name);
    debug!("path: {:#?}", path);
    read_log_file(&path).unwrap()
}

#[tauri::command(async)]
pub async fn import_zip(zip_path: String, app_handle: AppHandle<Wry>) {
    info!("Imporing modpack from {}", zip_path);
    let path = PathBuf::from(&zip_path);

    // Open the zip archive at `zip_path`
    let zip_file = File::open(&path).unwrap();
    let mut archive = ZipArchive::new(&zip_file).unwrap();

    match path.extension() {
        Some(extension) if extension == "zip" => import_curseforge_zip(&mut archive, &app_handle)
            .await
            .unwrap(),
        Some(extension) if extension == "mrpack" => import_modrinth_zip(&mut archive, &app_handle)
            .await
            .unwrap(),
        _ => {}
    }

    debug!("Invoked import_zip: {}", zip_path);
}

#[tauri::command(async)]
pub async fn get_curseforge_categories() -> Vec<CurseforgeCategory> {
    retrieve_curseforge_categories().await.unwrap()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackInformation {
    id: u32,
    name: String,
    summary: String,
    download_count: u32,
    authors: Vec<CurseforgeSearchAuthors>,
    logo: CurseforgeSearchImage,
    categories: Vec<CurseforgeCategory>,
}

impl From<CurseforgeSearchEntry> for ModpackInformation {
    fn from(value: CurseforgeSearchEntry) -> Self {
        let categories = value.get_basic_categories();
        Self {
            id: value.id,
            name: value.name,
            summary: value.summary,
            download_count: value.download_count,
            authors: value.authors,
            logo: value.logo,
            categories,
        }
    }
}

#[tauri::command(async)]
pub async fn search_curseforge(
    page: u32,
    search_filter: String,
    selected_version: String,
    selected_category: u32,
    selected_sort: String,
) -> Vec<ModpackInformation> {
    debug!("selected_sort: {}", selected_sort);
    let field = CurseforgeSortField::from(selected_sort);
    let version = if selected_version == "All Versions" {
        ""
    } else {
        selected_version.as_str()
    };
    debug!("Page: {}", page);
    debug!("selected_version: {}", selected_version);
    debug!("selected_category: {}", selected_category);

    let response =
        search_curseforge_modpacks(page, &search_filter, version, selected_category, field)
            .await
            .unwrap();

    debug!("Data: {:#?}", response.data.get(0));

    response
        .data
        .into_iter()
        .map(|entry| ModpackInformation::from(entry))
        .collect()
}
