use std::{collections::HashMap, thread, time::Instant};

use bytes::Bytes;
use futures::{executor::block_on, StreamExt};
use serde::{Deserialize, Deserializer};

// buffer_unordered(8) = 223768ms
// buffer_unordered(10) = 166119ms
// buffer_unordered(32) = 263426ms
#[tokio::main]
async fn main() {
    // main1().await;

    main2().await;
}

async fn main2() {
    let assets = block_on(download_object()).objects;

    const MAX_CHUNKS: usize = 4;
    let mut thread_handles = Vec::with_capacity(MAX_CHUNKS);
    let chunk_size = (assets.len() as f32 / MAX_CHUNKS as f32).ceil() as usize;

    let start = Instant::now();
    let chunks = assets.chunks(chunk_size);
    for asset in chunks {
        thread_handles.push(async move { download_all(asset).await });
    }
    futures::future::join_all(thread_handles).await;
    println!("Completed in {}ms", start.elapsed().as_millis());
}

async fn download_all(assets: &[Asset]) {
    let x = assets
        .into_iter()
        .map(|asset| {
            let first_two_chars = asset.hash.split_at(2);
            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                &first_two_chars.0, &asset.hash
            );
            download_bytes_from_url(url.clone())
        })
        .collect::<Vec<_>>();

    futures::stream::iter(x)
        // .buffer_unordered(8)
        .buffer_unordered(10)
        .collect::<Vec<reqwest::Result<Bytes>>>()
        .await;
}

async fn download_object() -> AssetObject {
    let url = "https://piston-meta.mojang.com/v1/packages/85615fd44408499f45e1cfe0cb3ea16227280a38/3.json";
    reqwest::get(url).await.unwrap().json().await.unwrap()
}

async fn main1() {
    let url = "https://piston-meta.mojang.com/v1/packages/85615fd44408499f45e1cfe0cb3ea16227280a38/3.json";
    let x: AssetObject = reqwest::get(url).await.unwrap().json().await.unwrap();
    println!("Here: {:#?}", x);

    let start = Instant::now();
    let mut futures = Vec::new();
    for item in x.objects {
        let first_two_chars = item.hash.split_at(2);
        let url = format!(
            "https://resources.download.minecraft.net/{}/{}",
            &first_two_chars.0, &item.hash
        );
        futures.push(download_bytes_from_url(url.clone()))
    }
    let all = futures::stream::iter(futures)
        .buffer_unordered(32)
        .collect::<Vec<reqwest::Result<Bytes>>>();

    all.await;
    println!("Completed in {}ms", start.elapsed().as_millis());
}

/// Download the bytes for a file at the specified `url`
pub async fn download_bytes_from_url(url: String) -> reqwest::Result<Bytes> {
    println!("Downloading {}...", url);
    // FIXME: If the http request fails, this just ignores it. We should be checking status codes.
    let client = reqwest::Client::new();
    let response = client.get(url).send().await.unwrap();
    response.bytes().await
}

#[derive(Debug, Deserialize)]
pub struct AssetObject {
    #[serde(deserialize_with = "to_asset_vec")]
    pub objects: Vec<Asset>,
}

fn to_asset_vec<'de, D>(deserializer: D) -> Result<Vec<Asset>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    struct TmpAsset {
        hash: String,
        size: u32,
    }

    let asset_map: HashMap<String, TmpAsset> = Deserialize::deserialize(deserializer)?;
    let mut result = Vec::with_capacity(asset_map.len());
    for (path, tmp_asset) in asset_map {
        result.push(Asset {
            path,
            hash: tmp_asset.hash,
            size: tmp_asset.size,
        });
    }
    Ok(result)
}

#[derive(Debug, Deserialize)]
pub struct Asset {
    path: String,
    hash: String,
    size: u32,
}
