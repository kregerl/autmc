use phf::phf_map;

pub const CLIENT_ID: &str = "94fd28d0-faa6-4d85-920d-69a2abe16bcd";
pub const SCOPE: &str = "XboxLive.signin offline_access";
pub const REDIRECT_URL: &str = "https://login.microsoftonline.com/common/oauth2/nativeclient";
pub const MICROSOFT_LOGIN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
pub const MICROSOFT_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
pub const XBOX_LIVE_AUTHENTICATE_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
pub const XTXS_AUTHENTICATE_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
pub const MINECRAFT_AUTHENTICATE_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
pub const MINECRAFT_LICENSE_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
pub const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

pub const VANILLA_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
pub const FORGE_MAVEN_BASE_URL: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge";
pub const FORGE_FILES_BASE_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge";
pub const FORGE_MANIFEST_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";
pub const FABRIC_BASE_URL: &str = "https://meta.fabricmc.net/v2";
/// The url to download assets from. Uses the hash as the endpoint: `...net/<first 2 hex letters of hash>/<whole hash>`
pub const VANILLA_ASSET_BASE_URL: &str = "https://resources.download.minecraft.net";
pub const JAVA_VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

pub const CURSEFORGE_API_URL: &str = "https://api.curseforge.com/v1";
pub const CURSEFORGE_MODPACK_CLASS_ID: u32 = 4471;
pub const CURSEFORGE_MODS_CLASS_ID: u32 = 6;
pub const CURSEFORGE_FORGECDN_URL: &str = "https://edge.forgecdn.net/files";

pub const LAUNCHER_NAME: &str = "Autmc";
pub const LAUNCHER_VERSION: &str = "1.0.0";

pub static XERR_HINTS: phf::Map<&'static str, &'static str> = phf_map! {
    "2148916233" => "2148916233: The account doesn't have an Xbox account. Once they sign up for one (or login through minecraft.net to create one) then they can proceed with the login. This shouldn't happen with accounts that have purchased Minecraft with a Microsoft account, as they would've already gone through that Xbox signup process.",
    "2148916235" => "2148916235: The account is from a country where Xbox Live is not available/banned",
    "2148916236" => "2148916236: The account needs adult verification on Xbox page. (South Korea)",
    "2148916237" => "2148916237: The account needs adult verification on Xbox page. (South Korea)",
    "2148916238" => "2148916238: The account is a child (under 18) and cannot proceed unless the account is added to a Family by an adult. This only seems to occur when using a custom Microsoft Azure application. When using the Minecraft launchers client id, this doesn't trigger."
};

pub const GZIP_SIGNATURE: [u8; 2] = [0x1f, 0x8b];