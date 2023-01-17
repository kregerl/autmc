use serde::Deserialize;



// #[derive(Debug, Deserialize)]
// pub struct FabricGameVersion {
//     pub version: String,
//     stable: bool,
// }

// #[derive(Debug, Deserialize)]
// pub struct FabricGameManifest(pub Vec<FabricGameVersion>);

#[derive(Debug, Deserialize)]
pub struct FabricLoaderVersion {
    separator: String,
    build: i32,
    maven: String,
    pub version: String,
    stable: bool
}

#[derive(Debug, Deserialize)]
pub struct FabricLoaderManifest(pub Vec<FabricLoaderVersion>);