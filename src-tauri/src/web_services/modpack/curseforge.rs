use crate::state::ManagerFromAppHandle;
use log::{debug, error, info};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::VecDeque,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    time::Instant,
};
#[cfg(test)]
use tauri::async_runtime::block_on;
use tauri::{AppHandle, Manager, State, Wry};
use zip::ZipArchive;

use crate::{
    consts::{
        CURSEFORGE_API_URL, CURSEFORGE_FORGECDN_URL, CURSEFORGE_MODPACK_CLASS_ID,
        CURSEFORGE_PAGE_SIZE,
    },
    state::instance_manager::{InstanceManager, InstanceState},
    web_services::{
        downloader::{
            buffered_download_stream, download_json_object, validate_hash_sha1, DownloadError,
            DownloadResult, Downloadable,
        },
        manifest::bytes_from_zip_file,
        resources::{create_instance, InstanceSettings, ModloaderType},
    },
};

// -----------------------------
// START: Curseforge Zip Files
// -----------------------------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeManifest {
    minecraft: CurseforgeGameInformation,
    // manifest_type: String,
    // manifest_version: u32,
    name: String,
    // version: String,
    author: String,
    files: Vec<CurseforgeFile>,
    overrides: String,
}

impl CurseforgeManifest {
    pub fn vanilla_version(&self) -> &str {
        &self.minecraft.version
    }

    pub fn modloaders(&self) -> &[Modloader] {
        &self.minecraft.modloaders
    }

    pub fn modpack_name(&self) -> &str {
        &self.name
    }

    pub fn overrides(&self) -> &str {
        &self.overrides
    }

    pub fn files(&self) -> &[CurseforgeFile] {
        &self.files
    }
}

#[derive(Debug, Deserialize)]
pub struct CurseforgeGameInformation {
    version: String,
    #[serde(rename = "modLoaders")]
    modloaders: Vec<Modloader>,
}

#[derive(Debug, Deserialize)]
pub struct Modloader {
    pub id: String,
    pub primary: bool,
}

#[derive(Debug, Deserialize)]
pub struct CurseforgeFile {
    // #[serde(rename = "projectID")]
    // project_id: u32,
    #[serde(rename = "fileID")]
    file_id: u32,
    // required: bool,
}

/// Extract the manifest from the curseforge zip.
pub fn extract_manifest_from_curseforge_zip(
    archive: &mut ZipArchive<&File>,
) -> io::Result<CurseforgeManifest> {
    info!("Extracting manifest from curseforge modpack zip");
    let manifest_bytes = bytes_from_zip_file(archive.by_name("manifest.json")?);

    Ok(serde_json::from_slice(&manifest_bytes)?)
}

/// Extract overrides into the instance's directory
pub fn extract_overrides(
    instance_path: &Path,
    archive: &mut ZipArchive<&File>,
    overrides: &str,
) -> io::Result<()> {
    info!("Extracting overrides into {:#?}", instance_path);
    for i in 0..archive.len() {
        let zip_file = archive.by_index(i)?;
        let name = zip_file.enclosed_name().unwrap().to_path_buf();
        if name.starts_with(overrides) && zip_file.is_file() {
            let timer = Instant::now();

            let base_path = name.strip_prefix(overrides).unwrap();
            let path = instance_path.join(base_path);
            let bytes = bytes_from_zip_file(zip_file);

            let parent = path.parent();
            if let Some(parent_dir) = parent {
                if !parent_dir.exists() {
                    fs::create_dir_all(parent_dir)?;
                }
            }
            let mut file = File::create(&path)?;
            file.write_all(&bytes)?;
            // TODO: speed up background.png extraction speed
            debug!(
                "Extracting {:#?} took {}ms for {} bytes",
                path,
                timer.elapsed().as_millis(),
                bytes.len()
            );
        }
    }
    Ok(())
}

// -----------------------------
// END: Curseforge Zip Files
// -----------------------------

// -----------------------------
// START: Common Curseforge Structs
// -----------------------------

#[derive(Debug, Deserialize)]
struct CurseforgeHash {
    value: String,
    // Valid hash algos: 1 = Sha1, 2 = Md5
    algo: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSortableGameVersion {
    game_version_name: String,
    game_version_padded: String,
    game_version: Option<String>,
    game_version_release_date: String,
    game_version_type_id: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeDependency {
    mod_id: u32,
    relation_type: u32,
}

#[derive(Debug, Deserialize)]
pub struct CurseforgeManifestInfo {
    pub instance_name: String,
    pub game_version: String,
    pub modloader_type: ModloaderType,
}

// -----------------------------
// END: Common Curseforge Structs
// -----------------------------

// -----------------------------
// START: Curseforge API Files Search
// -----------------------------

/// Download all mods from `files` into the instance's `mods` directory.
pub async fn download_mods_from_curseforge(
    files: &[CurseforgeFile],
    instances_dir: &Path,
    info: CurseforgeManifestInfo,
) -> DownloadResult<()> {
    info!("Requesting curseforge files");
    // Send request with headers and body content.
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "X-API-KEY",
        "$2a$10$5BgCleD8.rLQ5Ix17Xm2lOjgfoeTJV26a1BXmmpwrOemgI517.nuC"
            .parse()
            .unwrap(),
    );
    header_map.insert("Content-Type", "application/json".parse().unwrap());
    header_map.insert("Accept", "application/json".parse().unwrap());

    // extract just the file ids from `files`
    let file_ids: Vec<u32> = files.iter().map(|file| file.file_id).collect();

    let url = format!("{}/mods/files", CURSEFORGE_API_URL);
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .headers(header_map)
        .body(json!({ "fileIds": file_ids }).to_string())
        .send()
        .await?;

    let response = response.json::<CurseforgeFilesResponse>().await?;
    // Files to download.
    let mut download_vec: Vec<CurseforgeFilesData> = Vec::new();
    // Vec of dependencies to gather after processing manifest modids.
    let mut dependencies: Vec<u32> = Vec::new();

    // Store existing modids so dependencies can be skipped if already listed in the manifest.
    let existing_modids: Vec<_> = response.data.iter().map(|entry| entry.mod_id).collect();
    for files_data in response.data {
        for possible_dependency in &files_data.dependencies {
            // Possible enum values:
            // 1=EmbeddedLibrary
            // 2=OptionalDependency
            // 3=RequiredDependency
            // 4=Tool
            // 5=Incompatible
            // 6=Include
            // If we have a required dependency then add it to dependencies list.
            if possible_dependency.relation_type == 3
                && !existing_modids.contains(&possible_dependency.mod_id)
            {
                dependencies.push(possible_dependency.mod_id);
            }
        }
        download_vec.push(files_data);
    }

    info!("Gathering modids from {} dependencies", dependencies.len());
    for dependency_modid in dependencies {
        download_vec.extend(
            download_dependencies_recursively(
                &info.game_version,
                &info.modloader_type,
                dependency_modid,
            )
            .await?,
        );
    }

    let mods_dir = instances_dir.join(info.instance_name).join("mods");

    info!("Downloading {} mods from curseforge", download_vec.len());
    // Download all the files
    buffered_download_stream(&download_vec, &mods_dir, |bytes, file_data| {
        if !validate_hash_sha1(bytes, file_data.hash()) {
            let err = format!("Error downloading {}, invalid hash.", file_data.url());
            error!("{}", err);
            return Err(DownloadError::InvalidFileHash(err));
        }
        debug!("Downloading mod: {}", file_data.name());
        let path = file_data.path(&mods_dir);
        let mut file = File::create(path)?;
        file.write_all(bytes)?;
        Ok(())
    })
    .await?;

    Ok(())
}

/// Resursively download a mod and its dependencies at `modid`, filtered by `game_version` and `modloader_type`
#[async_recursion::async_recursion]
async fn download_dependencies_recursively(
    game_version: &str,
    modloader_type: &ModloaderType,
    modid: u32,
) -> reqwest::Result<Vec<CurseforgeFilesData>> {
    let mut dependencies = Vec::new();

    let search_entry = download_mod_from_modid(game_version, modloader_type, modid).await?;

    // If there is no entry response, then the modid doesn't exist or there is no file that matches the
    // `game_version` and `modloader_version` filters.
    match search_entry {
        Some(file_data) => {
            // If there are any required dependencies for this dependency, recurse.
            let required_dependencies = file_data
                .dependencies
                .iter()
                .filter(|dependency| dependency.relation_type == 3)
                .collect::<Vec<_>>();
            for required_dependency in required_dependencies {
                dependencies.extend(
                    download_dependencies_recursively(
                        game_version,
                        modloader_type,
                        required_dependency.mod_id,
                    )
                    .await?,
                );
            }

            dependencies.push(file_data)
        }
        None => {
            error!("File with modid {} could not be found", modid);
            debug!(
                "Filtering by game_version: {} and modloader_type: {}",
                game_version,
                modloader_type.to_string()
            );
        }
    }

    Ok(dependencies)
}

/// Download the file data about a given `modid`, filtered by `game_version` and `modloader_type` or
/// None if the `modid` doesn't exist
async fn download_mod_from_modid(
    game_version: &str,
    modloader_type: &ModloaderType,
    modid: u32,
) -> reqwest::Result<Option<CurseforgeFilesData>> {
    info!("Downloading mod file");
    let url = format!("{}/mods/{}/files", CURSEFORGE_API_URL, modid);
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "X-API-KEY",
        "$2a$10$5BgCleD8.rLQ5Ix17Xm2lOjgfoeTJV26a1BXmmpwrOemgI517.nuC"
            .parse()
            .unwrap(),
    );
    header_map.insert("Content-Type", "application/json".parse().unwrap());
    header_map.insert("Accept", "application/json".parse().unwrap());

    // Download a curseforge files response with files filtered to `game_version` and `modloader_version`
    let mut response: CurseforgeFilesResponse = download_json_object(
        &url,
        Some(header_map),
        Some(&[
            ("gameVersion", game_version),
            (
                "modLoaderVersion",
                modloader_id_from_version(modloader_type),
            ),
            // Without this sometimes versions with differing modloaders can be included.
            ("gameVersionTypeId", "6441"),
        ]),
    )
    .await?;

    // TODO: Sort by date?
    // Take the first element from data since they are already ordered by date and filtered during the request.
    Ok(response.data.pop_front())
}

/// Convert a [ModloaderType] to the `modLoaderVersion` query parameter
fn modloader_id_from_version(modloader_type: &ModloaderType) -> &str {
    match modloader_type {
        ModloaderType::Forge => "1",
        // Cauldron => 2
        // LiteLoader => 3
        ModloaderType::Fabric => "4",
        // Quilt => 5
        ModloaderType::None => "0",
    }
}

#[test]
fn test_download_mod_from_modid() {
    let x = block_on(download_mod_from_modid(
        "1.19.2",
        &ModloaderType::Forge,
        320926,
    ))
    .unwrap();
    println!("Here: {:#?}", x);
}

#[derive(Debug, Deserialize)]
struct CurseforgeFilesResponse {
    data: VecDeque<CurseforgeFilesData>,
    pagination: Option<CurseforgeSearchPagination>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeFilesData {
    id: u32,
    game_id: u32,
    mod_id: u32,
    is_available: bool,
    display_name: String,
    file_name: String,
    release_type: u8,
    file_status: u8,
    hashes: Vec<CurseforgeHash>,
    file_date: String,
    file_length: u32,
    download_count: u32,
    // This can be null when the mod author does not allow 3rd partys to automatically download the mod.
    download_url: Option<String>,
    game_versions: Vec<String>,
    sortable_game_versions: Vec<CurseforgeSortableGameVersion>,
    dependencies: Vec<CurseforgeDependency>,
    alternate_file_id: u32,
    is_server_pack: bool,
    file_fingerprint: u32,
    modules: Vec<CurseforgeModule>,
}

impl Downloadable for CurseforgeFilesData {
    fn name(&self) -> &str {
        &self.file_name
    }

    fn url(&self) -> String {
        match &self.download_url {
            Some(url) => url.into(),
            None => {
                let num_str = self.id.to_string();
                let parts = num_str.split_at(4);
                format!(
                    "{}/{}/{}/{}",
                    CURSEFORGE_FORGECDN_URL, parts.0, parts.1, self.file_name
                )
            }
        }
    }

    fn hash(&self) -> &str {
        &self
            .hashes
            .iter()
            .find(|hash| hash.algo == 1)
            .unwrap()
            .value
    }

    fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(&self.file_name)
    }
}

#[derive(Debug, Deserialize)]
struct CurseforgeModule {
    name: String,
    fingerprint: u32,
}

pub async fn import_curseforge_zip(
    mut archive: &mut ZipArchive<&File>,
    app_handle: &AppHandle<Wry>,
) -> io::Result<()> {
    // Pull out the manifest.json from the zip
    let curseforge_manifest = extract_manifest_from_curseforge_zip(&mut archive)?;

    let vanilla_version = curseforge_manifest.vanilla_version();
    let instance_name = curseforge_manifest.modpack_name();

    // Get the modloader with 'primary: true'
    let primary_modloader = curseforge_manifest
        .modloaders()
        .iter()
        .find(|modloader| modloader.primary);
    let (modloader_type, modloader_version) = match primary_modloader {
        Some(modloader) => {
            let splits = modloader.id.split('-').collect::<Vec<&str>>();
            (splits[0], splits[1])
        }
        None => {
            error!("Error getting primary modloader from manifest, does one exist?");
            ("", "")
        }
    };

    // Create corrected modloader version string for instance creation
    let full_modloader_version = format!("{}-{}", vanilla_version, modloader_version);

    let settings = InstanceSettings::new(
        instance_name.into(),
        vanilla_version.into(),
        modloader_type.into(),
        full_modloader_version,
        None,
    );

    create_instance(settings, &app_handle, Some(&curseforge_manifest.author))
        .await
        .unwrap();

    let instance_manager = InstanceManager::from_app_handle(&app_handle).await;

    let instances_dir = instance_manager.instances_dir();

    let info = CurseforgeManifestInfo {
        instance_name: instance_name.into(),
        game_version: curseforge_manifest.vanilla_version().into(),
        modloader_type: modloader_type.into(),
    };

    // After instance is created, download the mods from curseforge
    download_mods_from_curseforge(curseforge_manifest.files(), &instances_dir, info)
        .await
        .unwrap();

    // Finally extract overrides into the instance dir
    extract_overrides(
        &instances_dir.join(instance_name),
        &mut archive,
        curseforge_manifest.overrides(),
    )?;
    info!(
        "Succcessfully imported curseforge modpack {}",
        instance_name
    );
    Ok(())
}

// -----------------------------
// END: Curseforge API Files Search
// -----------------------------

// -----------------------------------------
// START: Curseforge API Mod/Modpack Search
// -----------------------------------------

#[derive(Debug, Deserialize)]
pub struct CurseforgeSearchResponse {
    pub data: Vec<CurseforgeSearchEntry>,
    pagination: CurseforgeSearchPagination,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeSearchEntry {
    pub id: u32,
    game_id: u32,
    pub name: String,
    slug: String,
    links: CurseforgeSearchEntryLinks,
    pub summary: String,
    status: u32,
    pub download_count: u32,
    is_featured: bool,
    primary_category_id: u32,
    categories: Vec<CurseforgeSearchCategory>,
    class_id: u32,
    pub authors: Vec<CurseforgeSearchAuthors>,
    pub logo: CurseforgeSearchImage,
    screenshots: Vec<CurseforgeSearchImage>,
    main_file_id: u32,
    latest_files: Vec<CurseforgeSearchLatestFiles>,
}

impl CurseforgeSearchEntry {
    pub fn get_basic_categories(&self) -> Vec<CurseforgeCategory> {
        self.categories
            .iter()
            .map(|category| CurseforgeCategory {
                id: category.id,
                name: category.name.to_owned(),
                icon_url: category.icon_url.to_owned(),
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSearchLatestFiles {
    id: u32,
    game_id: u32,
    mod_id: u32,
    is_available: bool,
    display_name: String,
    file_name: String,
    release_type: u32,
    file_status: u32,
    hashes: Vec<CurseforgeHash>,
    file_date: String,
    file_length: u32,
    download_count: u32,
    download_url: Option<String>,
    game_versions: Vec<String>,
    sortable_game_versions: Vec<CurseforgeSortableGameVersion>,
    dependencies: Vec<CurseforgeDependency>,
    alternate_file_id: u32,
    is_server_pack: bool,
    server_pack_file_id: Option<u32>,
    file_fingerprint: u32,
    modules: Vec<CurseforgeSearchModule>,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchModule {
    name: String,
    fingerprint: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeSearchImage {
    #[serde(skip_serializing)]
    id: u32,
    #[serde(skip_serializing)]
    mod_id: u32,
    title: String,
    #[serde(skip_serializing)]
    description: String,
    #[serde(skip_serializing)]
    thumbnail_url: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseforgeSearchAuthors {
    #[serde(skip_serializing)]
    id: u32,
    name: String,
    #[serde(skip_serializing)]
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSearchCategory {
    id: u32,
    game_id: u32,
    name: String,
    slug: String,
    url: String,
    icon_url: String,
    date_modified: String,
    is_class: bool,
    class_id: u32,
    parent_category_id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSearchEntryLinks {
    website_url: Option<String>,
    wiki_url: Option<String>,
    issues_url: Option<String>,
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSearchPagination {
    index: u32,
    page_size: u32,
    result_count: u32,
    total_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum CurseforgeSortField {
    Featured = 1,
    Popularity,
    LastUpdated,
    Name,
    Author,
    TotalDownloads,
    // Category,
    // GameVersion
}

impl From<String> for CurseforgeSortField {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Featured" => Self::Featured,
            "Popularity" => Self::Popularity,
            "LastUpdated" => Self::LastUpdated,
            "Name" => Self::Name,
            "Author" => Self::Author,
            "TotalDownloads" => Self::TotalDownloads,
            _ => unreachable!("Unknown sort field: {}", value),
        }
    }
}

impl CurseforgeSortField {
    pub fn as_number_str(self) -> String {
        (self as u8).to_string()
    }
}

pub async fn search_curseforge_modpacks(
    page: u32,
    search_filter: &str,
    selected_version: &str,
    selected_category: u32,
    selected_sort: CurseforgeSortField,
) -> reqwest::Result<CurseforgeSearchResponse> {
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "X-API-KEY",
        "$2a$10$5BgCleD8.rLQ5Ix17Xm2lOjgfoeTJV26a1BXmmpwrOemgI517.nuC"
            .parse()
            .unwrap(),
    );
    header_map.insert("Content-Type", "application/json".parse().unwrap());

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/mods/search", CURSEFORGE_API_URL))
        .headers(header_map)
        .query(&[
            ("gameId", "432"),
            ("classId", &CURSEFORGE_MODPACK_CLASS_ID.to_string()),
            ("categoryId", selected_category.to_string().as_str()),
            ("gameVersion", selected_version),
            ("searchFilter", search_filter),
            ("sortField", &selected_sort.as_number_str()),
            ("sortOrder", "desc"),
            ("index", &(page * CURSEFORGE_PAGE_SIZE).to_string()),
            ("pageSize", &CURSEFORGE_PAGE_SIZE.to_string()),
        ])
        .send()
        .await?;
    response.json::<CurseforgeSearchResponse>().await
}

#[test]
fn test_curseforge_search() {
    let x = block_on(search_curseforge_modpacks(
        1,
        "",
        "",
        4475,
        CurseforgeSortField::Popularity,
    ))
    .unwrap();
    // println!("Here: {:#?}", x);
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeCategory {
    id: u32,
    // game_id: u32,
    pub name: String,
    // slug: String,
    // url: String,
    icon_url: String,
    // date_modified: String,
    // class_id: u32,
    // parent_category_id: u32,
    // display_index: u32,
}

pub async fn retrieve_curseforge_categories() -> reqwest::Result<Vec<CurseforgeCategory>> {
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "X-API-KEY",
        "$2a$10$5BgCleD8.rLQ5Ix17Xm2lOjgfoeTJV26a1BXmmpwrOemgI517.nuC"
            .parse()
            .unwrap(),
    );
    header_map.insert("Content-Type", "application/json".parse().unwrap());

    #[derive(Deserialize)]
    struct Categories {
        data: Vec<CurseforgeCategory>,
    }

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/categories", CURSEFORGE_API_URL))
        .headers(header_map)
        .query(&[
            ("gameId", "432"),
            ("classId", &CURSEFORGE_MODPACK_CLASS_ID.to_string()),
        ])
        .send()
        .await?;
    Ok(response.json::<Categories>().await?.data)
}

#[test]
fn test_curseforge_categories() {
    let x = block_on(retrieve_curseforge_categories()).unwrap();
    println!("Here: {:#?}", x);
}

// -----------------------------------------
// END: Curseforge API Mod/Modpack Search
// -----------------------------------------
