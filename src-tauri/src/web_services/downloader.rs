use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use bytes::Bytes;
use crypto::{digest::Digest, sha1::Sha1, md5::Md5};
use futures::StreamExt;
use log::{debug, error, info};
use serde::de::DeserializeOwned;

const BUFFER_SIZE: usize = 8;

pub type DownloadResult<T> = Result<T, DownloadError>;

#[derive(Debug)]
pub enum DownloadError {
    RequestError(reqwest::Error),
    FileWriteError(io::Error),
    InvalidFileHashError(String),
}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        DownloadError::RequestError(err)
    }
}

impl From<io::Error> for DownloadError {
    fn from(error: io::Error) -> Self {
        DownloadError::FileWriteError(error)
    }
}

pub trait Downloadable {
    fn name(&self) -> &str;
    fn url(&self) -> String;
    fn hash(&self) -> &str;
    fn path(&self, base_dir: &Path) -> PathBuf;
}

pub async fn boxed_buffered_download_stream(
    items: &[Box<dyn Downloadable + Send + Sync>],
    base_dir: &Path,
    callback: impl Fn(&Bytes, &Box<dyn Downloadable + Send + Sync>) -> DownloadResult<()>,
) -> DownloadResult<()> {
    let mut futures = Vec::new();
    for item in items {
        futures.push(boxed_download_single(item, &base_dir, &callback));
    }
    let x = futures::stream::iter(futures)
        .buffer_unordered(BUFFER_SIZE)
        .collect::<Vec<DownloadResult<()>>>();

    x.await;
    Ok(())
}

async fn boxed_download_single(
    item: &Box<dyn Downloadable + Send + Sync>,
    base_dir: &Path,
    callback: impl Fn(&Bytes, &Box<dyn Downloadable + Send + Sync>) -> DownloadResult<()>,
) -> DownloadResult<()> {
    let path = &item.path(base_dir);
    if !path.exists() {
        debug!("Downloading file {}", item.name());
        let dir_path = path.parent().unwrap();
        fs::create_dir_all(dir_path)?;

        let bytes = download_bytes_from_url(&item.url()).await?;
        let x = callback(&bytes, item);
        if let Err(err) = x {
            // TODO: Implmenet display for error.
            error!("{:#?}", &err);
        }
    }
    Ok(())
}


pub async fn buffered_download_stream<T>(
    items: &[T],
    base_dir: &Path,
    callback: impl Fn(&Bytes, &T) -> DownloadResult<()>,
) -> DownloadResult<()>
where
    T: Downloadable,
{
    let mut futures = Vec::new();
    for item in items {
        futures.push(download_single(item, &base_dir, &callback));
    }
    let x = futures::stream::iter(futures)
        .buffer_unordered(BUFFER_SIZE)
        .collect::<Vec<DownloadResult<()>>>();

    x.await;
    Ok(())
}

async fn download_single<T>(
    item: &T,
    base_dir: &Path,
    callback: impl Fn(&Bytes, &T) -> DownloadResult<()>,
) -> DownloadResult<()>
where
    T: Downloadable,
{
    let path = &item.path(base_dir);
    if !path.exists() {
        debug!("Downloading file {}", item.name());
        let dir_path = path.parent().unwrap();
        fs::create_dir_all(dir_path)?;

        let bytes = download_bytes_from_url(&item.url()).await?;
        let x = callback(&bytes, item);
        if let Err(err) = x {
            // TODO: Implmenet display for error.
            error!("{:#?}", &err);
        }
    }
    Ok(())
}

pub async fn download_json_object<T>(url: &str) -> reqwest::Result<T>
where
    T: DeserializeOwned,
{
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    Ok(response.json().await?)
}

/// Download the bytes for a file at the specified `url`
pub async fn download_bytes_from_url(url: &str) -> reqwest::Result<Bytes> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    Ok(response.bytes().await?)
}

/// Validates that the hash of `bytes` matches the `valid_hash`
pub fn validate_hash(bytes: &Bytes, valid_hash: &str) -> bool {
    hash_bytes_sha1(bytes) == valid_hash
}

/// Hashes the `bytes` using SHA1 and returns the hex string
pub fn hash_bytes_sha1(bytes: &Bytes) -> String {
    let mut hasher = Sha1::new();
    hasher.input(bytes);
    hasher.result_str()
}

/// Hashes the `bytes` using MD5 and returns the hex string
pub fn hash_bytes_md5(bytes: &Bytes) -> String {
    let mut hasher = Md5::new();
    hasher.input(bytes);
    hasher.result_str()
}

/// Validates that the `path` exists and that the hash of it matches `valid_hash`
//TODO: Use this when a `strict` setting is enabled.
pub fn validate_file_hash(path: &Path, valid_hash: &str) -> bool {
    if !path.exists() {
        return false;
    }
    let result = read_bytes_from_file(path);
    if let Ok(bytes) = result {
        let valid = validate_hash(&bytes, &valid_hash);
        info!("REMOVEME: Is file valid: {}", valid);
        valid
    } else {
        false
    }
}

/// Reads and returns bytes from the file specified in `path`
fn read_bytes_from_file(path: &Path) -> io::Result<Bytes> {
    let mut file = File::open(&path)?;
    let metadata = file.metadata()?;
    let mut buffer = vec![0; metadata.len() as usize];
    file.read(&mut buffer)?;
    Ok(Bytes::from(buffer))
}
