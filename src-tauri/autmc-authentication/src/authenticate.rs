use crate::{
    consts::{
        CLIENT_ID, DEVICE_CODE_GRANT_TYPE, DEVICE_CODE_SCOPE, MICROSOFT_DEVICE_CODE_URL,
        MICROSOFT_TOKEN_URL, MINECRAFT_AUTHENTICATE_URL, MINECRAFT_PROFILE_URL,
        XBOX_LIVE_AUTHENTICATE_URL, XTXS_AUTHENTICATE_URL,
    },
    error::{
        AuthenticationError, AuthenticationResult, MicrosoftErrorResponse,
        MincraftProfileErrorResponse, MinecraftTokenErrorResponse, XboxErrorResponse,
    },
};
use autmc_log::debug_if;
use log::debug;
use reqwest::{Client, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, thread::sleep, time::Duration};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MinecraftAccount {
    pub uuid: String,
    pub name: String,
    // FIXME: Cache downloaded skins instead of saving url to download everytime.
    pub skin_url: String,
    pub microsoft_access_token: String,
    pub microsoft_access_token_expiry: u64,
    pub microsoft_refresh_token: String,
    pub minecraft_access_token: String,
    pub minecraft_access_token_expiry: u64,
}

impl Into<MicrosoftToken> for MinecraftAccount {
    fn into(self) -> MicrosoftToken {
        MicrosoftToken {
            access_token: self.microsoft_access_token,
            refresh_token: self.microsoft_refresh_token,
            access_token_expiry: self.microsoft_access_token_expiry,
        }
    }
}

impl MinecraftAccount {
    fn new(
        minecraft_profile_response: MinecraftProfileResponse,
        microsoft_token: MicrosoftToken,
        minecraft_token_response: MinecraftTokenResponse,
    ) -> Self {
        let skin_url = minecraft_profile_response.active_skin().url.clone();

        let minecraft_access_token_expiry = (chrono::Local::now().timestamp()
            + (minecraft_token_response.expires_in as i64)
            - 10) as u64;
        Self {
            uuid: minecraft_profile_response.id,
            name: minecraft_profile_response.name,
            skin_url,
            microsoft_access_token: microsoft_token.access_token,
            microsoft_access_token_expiry: microsoft_token.access_token_expiry,
            microsoft_refresh_token: microsoft_token.refresh_token,
            minecraft_access_token: minecraft_token_response.access_token,
            minecraft_access_token_expiry,
        }
    }
}

#[derive(Debug, Deserialize)]
/// Response struct for the Microsoft OAuth process.  
/// Commented out fields are currenty unused but exist in the response
struct MicrosoftTokenResponse {
    // token_type: String,
    // scope: String,
    expires_in: u32,
    // ext_expires_in: u32,
    access_token: String,
    refresh_token: String,
}

impl Into<MicrosoftToken> for MicrosoftTokenResponse {
    fn into(self) -> MicrosoftToken {
        let access_token_expiry =
            (chrono::Local::now().timestamp() + (self.expires_in as i64) - 10) as u64;

        MicrosoftToken {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            access_token_expiry,
        }
    }
}

#[derive(Debug)]
pub struct MicrosoftToken {
    access_token: String,
    refresh_token: String,
    access_token_expiry: u64,
}

#[derive(Debug)]
pub enum OAuthRefreshMode {
    Microsoft { refresh_token: String },
    Minecraft { token: MicrosoftToken },
}

pub async fn refresh_access_tokens(
    refresh_mode: OAuthRefreshMode,
) -> AuthenticationResult<MinecraftAccount> {
    let microsoft_token = match refresh_mode {
        OAuthRefreshMode::Microsoft { refresh_token } => {
            let microsoft_token_response = refresh_microsoft_token(&refresh_token).await?;
            microsoft_token_response.into()
        }
        OAuthRefreshMode::Minecraft { token } => token,
    };

    continue_authentication_flow(microsoft_token).await
}

pub async fn start_device_code_authentication() -> AuthenticationResult<DeviceCode> {
    debug!("Requesting Microsoft device code authentication format.");
    let device_code_response = get_microsoft_devicecode().await?;
    debug_if!(
        "AUTHENTICATION",
        "Received user code '{}' and device code token '{}'",
        device_code_response.user_code,
        device_code_response.device_code
    );

    Ok(device_code_response.into())
}

pub async fn poll_device_code_status(device_code: &str) -> AuthenticationResult<MinecraftAccount> {
    // Maximum number of attempts, each attempts will sleep for 1s
    const MAX_ATTEMPTS: usize = 120;
    let mut attempts = 0;
    debug!("Polling OAuth device code endpoint");
    let microsoft_token_response = loop {
        debug_if!(
            "AUTHENTICATION",
            "Attempt #{} while polling device code endpoint.",
            attempts
        );
        if attempts >= MAX_ATTEMPTS {
            return Err(AuthenticationError::MaxAttemptsExceeded(
                "Device code authentication took longer than 2 minutes.".into(),
            ));
        }

        let token_response = poll_microsoft_token_endpoint(device_code).await?;
        if !token_response.status().is_success() {
            sleep(Duration::from_secs(1));
            attempts += 1;
        } else {
            break get_response_if_ok::<MicrosoftTokenResponse, MicrosoftErrorResponse>(
                token_response,
            )
            .await?;
        }
    };
    debug_if!(
        "AUTHENTICATION",
        "Received Microsoft access token '{}'",
        microsoft_token_response.access_token
    );
    continue_authentication_flow(microsoft_token_response.into()).await
}

async fn continue_authentication_flow(
    microsoft_token: MicrosoftToken,
) -> AuthenticationResult<MinecraftAccount> {
    debug!("Requesting XBox Live access token.");
    let xbl_token_response = get_xbl_token(&microsoft_token.access_token).await?;
    debug_if!(
        "AUTHENTICATION",
        "Received XBox Live access token '{}'",
        xbl_token_response.access_token
    );

    debug!("Requesting Xbox Secure Token Service access token.");
    let xsts_token_response = get_xsts_token(&xbl_token_response.access_token).await?;
    debug_if!(
        "AUTHENTICATION",
        "Received Xbox Secure Token Service access token '{}'",
        xsts_token_response.access_token
    );
    let user_hash = match xsts_token_response.get_user_hash() {
        Some(user_hash) => user_hash,
        None => return Err(AuthenticationError::XSTSMissingUserHash),
    };

    debug!("Requesting Minecraft access token.");
    let minecraft_token_response =
        get_minecraft_token(&xsts_token_response.access_token, &user_hash).await?;
    debug_if!(
        "AUTHENTICATION",
        "Received Minecraft access token '{}'",
        minecraft_token_response.access_token
    );
    // NOTE: Since Xbox Game Pass users don't technically own the game, the entitlement endpoint will show as such.
    // It should be used to check the official public key from liblauncher.so but whats the point in checking if
    // a user owns the game before attempting the next step, if it won't work for Xbox Game Pass users anyway?
    // let _ = check_license(&minecraft_token_response.access_token).await?;

    debug!("Requesting Minecraft profile.");
    let mincraft_profile_response =
        get_minecraft_profile(&minecraft_token_response.access_token).await?;
    debug_if!(
        "AUTHENTICATION",
        "Received Minecraft profile for '{}'",
        mincraft_profile_response.name
    );

    let account = MinecraftAccount::new(
        mincraft_profile_response,
        microsoft_token,
        minecraft_token_response,
    );
    Ok(account)
}

#[derive(Debug, Deserialize)]
/// Response struct for the Microsoft DevicCode polling process.  
/// Commented out fields are currenty unused but exist in the response
struct DeviceCodeResponse {
    user_code: String,
    device_code: String,
    // verification_uri: String,
    // expires_in: u32,
    // interval: u32,
    message: String,
}

impl Into<DeviceCode> for DeviceCodeResponse {
    fn into(self) -> DeviceCode {
        DeviceCode {
            message: self.message,
            device_code: self.device_code,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DeviceCode {
    pub message: String,
    pub device_code: String,
}

async fn get_microsoft_devicecode() -> AuthenticationResult<DeviceCodeResponse> {
    let client = Client::new();
    let response = client
        .get(MICROSOFT_DEVICE_CODE_URL)
        .query(&[CLIENT_ID, DEVICE_CODE_SCOPE])
        .send()
        .await?;

    get_response_if_ok::<DeviceCodeResponse, MicrosoftErrorResponse>(response).await
}

async fn poll_microsoft_token_endpoint(device_code: &str) -> AuthenticationResult<Response> {
    let mut form: HashMap<&str, &str> = HashMap::new();
    form.insert("device_code", device_code);
    form.insert(DEVICE_CODE_GRANT_TYPE.0, DEVICE_CODE_GRANT_TYPE.1);
    form.insert(CLIENT_ID.0, CLIENT_ID.1);

    let client = Client::new();
    Ok(client.post(MICROSOFT_TOKEN_URL).form(&form).send().await?)
}

async fn refresh_microsoft_token(
    refresh_token: &str,
) -> AuthenticationResult<MicrosoftTokenResponse> {
    let mut form: HashMap<&str, &str> = HashMap::new();
    form.insert(CLIENT_ID.0, CLIENT_ID.1);
    form.insert(DEVICE_CODE_SCOPE.0, DEVICE_CODE_SCOPE.1);
    form.insert("grant_type", "refresh_token");
    form.insert("refresh_token", refresh_token);

    let client = Client::new();
    let response = client.post(MICROSOFT_TOKEN_URL).form(&form).send().await?;
    get_response_if_ok::<MicrosoftTokenResponse, MicrosoftErrorResponse>(response).await
}

#[derive(Debug, Deserialize)]
/// Response struct for the XBox Live authentication process.  
/// Commented out fields are currenty unused but exist in the response
pub struct XboxTokenResponse {
    // #[serde(rename = "IssueInstant")]
    // issue_instant: String,
    // #[serde(rename = "NotAfter")]
    // not_after: String,
    #[serde(rename = "Token")]
    access_token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: HashMap<String, Vec<HashMap<String, String>>>,
}

impl XboxTokenResponse {
    pub fn get_user_hash(&self) -> Option<String> {
        let xui = self.display_claims.get("xui")?;
        let uhs = xui.first()?.get("uhs")?;
        Some(uhs.into())
    }
}

/// Sends request to the XboxLive `/authenticate` endpoint using a Microsoft access token
async fn get_xbl_token(access_token: &str) -> AuthenticationResult<XboxTokenResponse> {
    let client = reqwest::Client::new();
    let response = client
        .post(XBOX_LIVE_AUTHENTICATE_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(
            json!({
                "Properties": {
                    "AuthMethod": "RPS",
                    "SiteName": "user.auth.xboxlive.com",
                    "RpsTicket": format!("d={}", access_token)
                },
                "RelyingParty": "http://auth.xboxlive.com",
                "TokenType": "JWT"
            })
            .to_string(),
        )
        .send()
        .await?;
    get_response_if_ok::<XboxTokenResponse, XboxErrorResponse>(response).await
}

/// Sends request to the Xbox Secure Token Service `/authorize` endpoint using an XboxLive access token
async fn get_xsts_token(xbl_token: &str) -> AuthenticationResult<XboxTokenResponse> {
    let client = reqwest::Client::new();
    let response = client
        .post(XTXS_AUTHENTICATE_URL)
        .body(
            json!({
                "Properties": {
                    "SandboxId": "RETAIL",
                    "UserTokens": [
                        xbl_token
                    ]
                },
                "RelyingParty": "rp://api.minecraftservices.com/",
                "TokenType": "JWT"
            })
            .to_string(),
        )
        .send()
        .await?;
    get_response_if_ok::<XboxTokenResponse, XboxErrorResponse>(response).await
}

#[derive(Debug, Deserialize)]
/// Response struct for the Minecraft authentication process.  
/// Commented out fields are currenty unused but exist in the response
pub struct MinecraftTokenResponse {
    // This is not the uuid of the mc account
    // username: String,
    access_token: String,
    expires_in: u32,
    // token_type: String,
}

/// Sends request to the mojang `/login_with_xbox` endpoint using the user hash and XSTS token
async fn get_minecraft_token(
    xsts_token: &str,
    user_hash: &str,
) -> AuthenticationResult<MinecraftTokenResponse> {
    let client = reqwest::Client::new();
    let response = client
        .post(MINECRAFT_AUTHENTICATE_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(
            json!({
                "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_token),
                "ensureLegacyEnabled": true
            })
            .to_string(),
        )
        .send()
        .await?;

    get_response_if_ok::<MinecraftTokenResponse, MinecraftTokenErrorResponse>(response).await
}

// /// Unused for now, currently cannot show if a Xbox Game Pass user owns the game so whats the point in checking...
// async fn check_license(access_token: &str) -> AuthenticationResult<()> {
//     let client = reqwest::Client::new();
//     let response = client
//         .get(MINECRAFT_LICENSE_URL)
//         .header("Content-Type", "application/json")
//         .header("Accept", "application/json")
//         .header("Authorization", format!("Bearer {}", access_token))
//         .send()
//         .await?;

//     Ok(())
// }

// TODO: Save the entire skin struct in the accounts file instead of just the URL.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MinecraftProfileSkin {
    id: String,
    state: String,
    url: String,
    variant: String,
    alias: Option<String>,
}

#[derive(Debug, Deserialize)]
/// Response struct for the Minecraft profile request.  
struct MinecraftProfileResponse {
    id: String,
    name: String,
    skins: Vec<MinecraftProfileSkin>,
}

impl MinecraftProfileResponse {
    /// It is assumed that at least one of the skins will have state "ACTIVE"
    /// If it is possible, we default to the first skin in the list if there is no active skin.
    pub fn active_skin(&self) -> &MinecraftProfileSkin {
        for skin in &self.skins {
            if skin.state == "ACTIVE" {
                return skin;
            }
        }
        // Unwrap here since it should be impossible to get an empty vec of skins.
        self.skins.get(0).unwrap()
    }
}

/// Obtains the Minecraft profile information like uuid, username, skins, and capes
async fn get_minecraft_profile(
    access_token: &str,
) -> AuthenticationResult<MinecraftProfileResponse> {
    let client = reqwest::Client::new();
    let response = client
        .get(MINECRAFT_PROFILE_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;

    get_response_if_ok::<MinecraftProfileResponse, MincraftProfileErrorResponse>(response).await
}

/// Deserialize the response into `T` if the status is 200 OK  
/// Otherwise attempt to deserialize into the error response struct `E`  
///
/// If all else fails, return a generic HTTP error containing the error code.
async fn get_response_if_ok<T, E>(response: Response) -> AuthenticationResult<T>
where
    T: DeserializeOwned,
    E: DeserializeOwned,
    AuthenticationError: From<E>,
{
    let status = response.status();
    if status.is_success() {
        Ok(response.json::<T>().await?)
    } else {
        match response.content_length() {
            Some(content_length) if content_length > 0 => {
                Err(AuthenticationError::from(response.json::<E>().await?))
            }
            _ => Err(AuthenticationError::HttpResponseError(status)),
        }
    }
}
