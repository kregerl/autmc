use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde_json::json;
use tauri::{AppHandle, Wry};
#[cfg(test)]
use tauri::async_runtime::block_on;
use zip::{result::ZipError, ZipArchive};

use crate::{consts::CURSEFORGE_API_URL, web_services::manifest::bytes_from_zip_file};

#[derive(Debug, Deserialize)]
pub struct CurseforgeManifest {
    minecraft: CurseforgeGameInformation,
    #[serde(rename = "manifestType")]
    manifest_type: String,
    #[serde(rename = "manifestVersion")]
    manifest_version: u32,
    name: String,
    version: String,
    author: String,
    pub files: Vec<CurseforgeFile>,
    overrides: String,
}

impl CurseforgeManifest {
    pub fn get_vanilla_version(&self) -> &str {
        &self.minecraft.version
    }

    pub fn get_modloaders(&self) -> &[Modloader] {
        &self.minecraft.modloaders
    }

    pub fn get_modpack_name(&self) -> &str {
        &self.name
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
    #[serde(rename = "projectID")]
    project_id: u32,
    #[serde(rename = "fileID")]
    file_id: u32,
    required: bool,
}

pub fn deserialize_curseforge_zip(path: &Path) -> Result<CurseforgeManifest, io::Error> {
    let zip_file = File::open(&path)?;
    let mut archive = ZipArchive::new(&zip_file)?;

    let manifest_bytes = bytes_from_zip_file(archive.by_name("manifest.json")?);
    Ok(serde_json::from_slice(&manifest_bytes)?)
}

pub async fn download_mods_from_curseforge(
    files: &[CurseforgeFile],
    instance_name: &str,
    app_handle: &AppHandle<Wry>,
) -> reqwest::Result<()> {
    let mut header_map = HeaderMap::new();
    // TODO: Get own API-key
    header_map.insert(
        "X-API-KEY",
        "$2a$10$5BgCleD8.rLQ5Ix17Xm2lOjgfoeTJV26a1BXmmpwrOemgI517.nuC"
            .parse()
            .unwrap(),
    );
    header_map.insert("Content-Type", "application/json".parse().unwrap());
    header_map.insert("Accept", "application/json".parse().unwrap());

    let file_ids: Vec<u32> = files.into_iter().map(|file| file.file_id).collect();

    let url = format!("{}/mods/files", CURSEFORGE_API_URL);
    let client = reqwest::Client::new();
    let response = client.post(url)
    .headers(header_map)
    .body(
        json!({
            "fileIds": file_ids 
        })
        .to_string(),
    ).send().await?;
    
    let result = response.json::<CurseforgeSearchResult>().await?;
    println!("result: {:#?}", result);
    Ok(())
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchResult {
    data: Vec<CurseforgeSearchEntry>,
    pagination: CurseforgeSearchPagination,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchEntry {
    id: u32,
    #[serde(rename = "gameId")]
    game_id: u32,
    name: String,
    slug: String,
    links: CurseforgeSearchEntryLinks,
    summary: String,
    status: u32,
    #[serde(rename = "downloadCount")]
    download_count: u32,
    #[serde(rename = "isFeatured")]
    is_featured: bool,
    #[serde(rename = "primaryCategoryId")]
    primary_category_id: u32,
    categories: Vec<CurseforgeSearchCategory>,
    #[serde(rename = "classId")]
    class_id: u32,
    authors: Vec<CurseforgeSearchAuthors>,
    logo: CurseforgeSearchImage,
    screenshots: Vec<CurseforgeSearchImage>,
    #[serde(rename = "mainFileId")]
    main_file_id: u32,
    #[serde(rename = "latestFiles")]
    latest_files: Vec<CurseforgeSearchLatestFiles>,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchLatestFiles {
    id: u32,
    #[serde(rename = "gameId")]
    game_id: u32,
    #[serde(rename = "modId")]
    mod_id: u32,
    #[serde(rename = "isAvailable")]
    is_available: bool,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "releaseType")]
    release_type: u32,
    #[serde(rename = "fileStatus")]
    file_status: u32,
    hashes: Vec<CurseforgeSearchHash>,
    #[serde(rename = "fileDate")]
    file_date: String,
    #[serde(rename = "fileLength")]
    file_length: u32,
    #[serde(rename = "downloadCount")]
    download_count: u32,
    #[serde(rename = "downloadUrl")]
    download_url: Option<String>,
    #[serde(rename = "gameVersions")]
    game_versions: Vec<String>,
    #[serde(rename = "sortableGameVersions")]
    sortable_game_versions: Vec<CurseforgeSortableGameVersion>,
    dependencies: Vec<CurseforgeSearchDependency>,
    #[serde(rename = "alternateFileId")]
    alternate_file_id: u32,
    #[serde(rename = "isServerPack")]
    is_server_pack: bool,
    #[serde(rename = "serverPackFileId")]
    server_pack_file_id: Option<u32>,
    #[serde(rename = "fileFingerprint")]
    file_fingerprint: u32,
    modules: Vec<CurseforgeSearchModule>,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchModule {
    name: String,
    fingerprint: u32,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchDependency {
    #[serde(rename = "modId")]
    mod_id: u32,
    #[serde(rename = "relationType")]
    relation_type: u32,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSortableGameVersion {
    #[serde(rename = "gameVersionName")]
    game_version_name: String,
    #[serde(rename = "gameVersionPadded")]
    game_version_padded: String,
    #[serde(rename = "gameVersion")]
    game_version: String,
    #[serde(rename = "gameVersionReleaseDate")]
    game_version_release_date: String,
    #[serde(rename = "gameVersionTypeId")]
    game_version_type_id: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchHash {
    value: String,
    // Valid hash algos: 1 = Sha1, 2 = Md5
    algo: u8,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchImage {
    id: u32,
    #[serde(rename = "modId")]
    mod_id: u32,
    title: String,
    description: String,
    #[serde(rename = "thumbnailUrl")]
    thumbnail_url: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchAuthors {
    id: u32,
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchCategory {
    id: u32,
    #[serde(rename = "gameId")]
    game_id: u32,
    name: String,
    slug: String,
    url: String,
    #[serde(rename = "iconUrl")]
    icon_url: String,
    #[serde(rename = "dateModified")]
    date_modified: String,
    #[serde(rename = "isClass")]
    is_class: bool,
    #[serde(rename = "classId")]
    class_id: u32,
    #[serde(rename = "parentCategoryId")]
    parent_category_id: u32,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchEntryLinks {
    #[serde(rename = "websiteUrl")]
    website_url: Option<String>,
    #[serde(rename = "wikiUrl")]
    wiki_url: Option<String>,
    #[serde(rename = "issuesUrl")]
    issues_url: Option<String>,
    #[serde(rename = "sourceUrl")]
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchPagination {
    index: u32,
    #[serde(rename = "pageSize")]
    page_size: u32,
    #[serde(rename = "resultCount")]
    result_count: u32,
    #[serde(rename = "totalCount")]
    total_count: u32,
}

async fn search_curseforge_modpacks() -> reqwest::Result<CurseforgeSearchResult> {
    let mut header_map = HeaderMap::new();
    // TODO: Get own API-key
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
            ("categoryId", "0"),
            ("pageSize", "40"),
            ("index", "0"),
            ("sortField", "1"),
            ("sortOrder", "desc"),
            ("classId", "4471"),
        ])
        .send()
        .await?;
    response.json::<CurseforgeSearchResult>().await
}

#[test]
fn test_curseforge_search() {
    let x = block_on(search_curseforge_modpacks()).unwrap();
    println!("Here: {:#?}", x);
}
