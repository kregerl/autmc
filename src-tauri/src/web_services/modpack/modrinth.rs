use serde::Deserialize;


#[derive(Debug, Deserialize)]
struct ModrinthManifest {
    #[serde(rename = "formatVersion")]
    format_version: u32,
    game: String,
    #[serde(rename = "versionId")]
    version_id: String,
    name: String,
    summary: Option<String>,
    files: Vec<ModrinthFile>,
    dependencies: ModrinthDependencies,
}

#[derive(Debug, Deserialize)]
struct ModrinthFile {
    path: String,
    hashes: ModrinthHashes,
    // Vec of all possible downloads, if one fails fallback to others
    env: Option<ModrinthEnv>,
    downloads: Vec<String>,
    #[serde(rename = "fileSize")]
    file_size: u32
}

#[derive(Debug, Deserialize)]
struct ModrinthHashes {
    sha1: String,
    sha512: String,
}

#[derive(Debug, Deserialize)]
struct ModrinthEnv {
    client: String,
    server: String,
}

#[derive(Debug, Deserialize)]
struct ModrinthDependencies {
    minecraft: String,
    #[serde(rename = "fabric-loader")]
    fabric: Option<String>,
    forge: Option<String>,
}