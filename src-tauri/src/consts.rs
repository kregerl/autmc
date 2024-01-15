use phf::phf_map;

pub const VANILLA_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
pub const FORGE_MAVEN_BASE_URL: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge";
pub const FORGE_FILES_BASE_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge";
pub const FORGE_MANIFEST_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";
pub const FABRIC_BASE_URL: &str = "https://meta.fabricmc.net/v2";
/// The url to download assets from. Uses the hash as the endpoint: `...net/<first 2 hex letters of hash>/<whole hash>`
pub const VANILLA_ASSET_BASE_URL: &str = "https://resources.download.minecraft.net";
pub const JAVA_VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

pub const MINECRAFT_LIBRARIES_URL: &str = "https://libraries.minecraft.net"; 

pub const CURSEFORGE_API_URL: &str = "https://api.curseforge.com/v1";
pub const CURSEFORGE_MODPACK_CLASS_ID: u32 = 4471;
pub const CURSEFORGE_MODS_CLASS_ID: u32 = 6;
pub const CURSEFORGE_FORGECDN_URL: &str = "https://edge.forgecdn.net/files";
pub const CURSEFORGE_PAGE_SIZE: u32 = 50;


pub const LAUNCHER_NAME: &str = "Autmc";
pub const LAUNCHER_VERSION: &str = "1.0.0";

pub const GZIP_SIGNATURE: [u8; 2] = [0x1f, 0x8b];