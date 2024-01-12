use phf::phf_map;

pub(crate) const CLIENT_ID: (&str, &str) = ("client_id", "94fd28d0-faa6-4d85-920d-69a2abe16bcd");
pub(crate) const DEVICE_CODE_SCOPE: (&str, &str) = ("scope", "XboxLive.signin offline_access");
pub(crate) const DEVICE_CODE_GRANT_TYPE: (&str, &str) = ("grant_type", "urn:ietf:params:oauth:grant-type:device_code");
pub(crate) const MICROSOFT_DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
pub(crate) const MICROSOFT_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
pub(crate) const XBOX_LIVE_AUTHENTICATE_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
pub(crate) const XTXS_AUTHENTICATE_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
pub(crate) const MINECRAFT_AUTHENTICATE_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
// pub(crate) const MINECRAFT_LICENSE_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
pub(crate) const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";


pub(crate) static XERR_HINTS: phf::Map<&'static str, &'static str> = phf_map! {
    "2148916233" => "2148916233: The account doesn't have an Xbox account. Once they sign up for one (or login through minecraft.net to create one) then they can proceed with the login. This shouldn't happen with accounts that have purchased Minecraft with a Microsoft account, as they would've already gone through that Xbox signup process.",
    "2148916235" => "2148916235: The account is from a country where Xbox Live is not available/banned",
    "2148916236" => "2148916236: The account needs adult verification on Xbox page. (South Korea)",
    "2148916237" => "2148916237: The account needs adult verification on Xbox page. (South Korea)",
    "2148916238" => "2148916238: The account is a child (under 18) and cannot proceed unless the account is added to a Family by an adult. This only seems to occur when using a custom Microsoft Azure application. When using the Minecraft launchers client id, this doesn't trigger."
};