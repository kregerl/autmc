use serde::Deserialize;
use sha1::{Sha1, Digest};

const VANILLA_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Deserialize)]
struct VanillaLatest {
    release: String,
    snapshot: String,
}

#[derive(Debug, Deserialize)]
struct VanillaVersion {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
    time: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
    sha1: String,
    #[serde(rename = "complianceLevel")]
    compliance_level: u32
}

#[derive(Debug, Deserialize)]
struct VanillaManifest {
    latest: VanillaLatest,
    versions: Vec<VanillaVersion>,
}

type ManifestResult<T> = Result<T, ManifestError>;

pub enum ManifestError {
    HttpError(reqwest::Error),
}

impl From<reqwest::Error> for ManifestError {
    fn from(e: reqwest::Error) -> Self {
        ManifestError::HttpError(e)
    }
}

pub async fn obtain_vanilla_manifest() ->  ManifestResult<()> {
    let client = reqwest::Client::new();
    let response = client
    .get(VANILLA_MANIFEST_URL)
    .send()
    .await?;

    let json = response.json::<VanillaManifest>().await?;
    println!("Response: {:#?}", json);
    // let x = validate_hash(json.versions.first().unwrap()).await?;
    Ok(())
}

// async fn validate_hash(version: &VanillaVersion) -> ManifestResult<()> {
//     let mut hasher = Sha1::new();
//     hasher.update(response.text().await?);
//     let result = hasher.finalize();
//     println!("Hasher Result: {:#?}", result);

//     Ok(())
// }