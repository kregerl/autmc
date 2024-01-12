use std::{collections::HashMap, thread::sleep, time::Duration};

use reqwest::{Client, Response};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::json;

use crate::{
    consts::{
        CLIENT_ID, DEVICE_CODE_GRANT_TYPE, DEVICE_CODE_SCOPE, MICROSOFT_DEVICE_CODE_URL,
        MICROSOFT_TOKEN_URL, MINECRAFT_AUTHENTICATE_URL, MINECRAFT_PROFILE_URL,
        XBOX_LIVE_AUTHENTICATE_URL, XTXS_AUTHENTICATE_URL,
    },
    error::{
        AuthenticationError, AuthenticationResult, MicrosoftErrorResponse,
        MincraftProfileErrorResponse, XboxErrorResponse,
    },
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct MinecraftAccount {
    uuid: String,
    name: String,
    // FIXME: Cache downloaded skins instead of saving url to download everytime.
    skin_url: String,
    microsoft_access_token: String,
    microsoft_access_token_expiry: u64,
    microsoft_refresh_token: String,
    minecraft_access_token: String,
    minecraft_access_token_expiry: u64,
}

impl MinecraftAccount {
    fn new(
        minecraft_profile_response: MinecraftProfileResponse,
        microsoft_token_response: MicrosoftTokenResponse,
        minecraft_token_response: MinecraftTokenResponse,
    ) -> Self {
        let skin_url = minecraft_profile_response.active_skin().url.clone();

        let microsoft_access_token_expiry = (chrono::Local::now().timestamp()
            + (microsoft_token_response.expires_in as i64)
            - 10) as u64;

        let minecraft_access_token_expiry = (chrono::Local::now().timestamp()
            + (minecraft_token_response.expires_in as i64)
            - 10) as u64;
        Self {
            uuid: minecraft_profile_response.id,
            name: minecraft_profile_response.name,
            skin_url,
            microsoft_access_token: microsoft_token_response.access_token,
            microsoft_access_token_expiry,
            microsoft_refresh_token: microsoft_token_response.refresh_token,
            minecraft_access_token: minecraft_token_response.access_token,
            minecraft_access_token_expiry,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MicrosoftTokenResponse {
    token_type: String,
    scope: String,
    expires_in: u32,
    ext_expires_in: u32,
    access_token: String,
    refresh_token: String,
}

pub async fn authenticate_with_device_code() -> AuthenticationResult<MinecraftAccount> {
    let device_code_response = get_microsoft_devicecode().await?;

    println!("{}", device_code_response.message);

    // Maximum number of attempts, each attempts will sleep for 1s
    const MAX_ATTEMPTS: usize = 120;
    let mut attempts = 0;
    let microsoft_token_response = loop {
        if attempts >= MAX_ATTEMPTS {
            return Err(AuthenticationError::MaxAttemptsExceeded);
        }

        let token_response =
            poll_microsoft_token_endpoint(&device_code_response.device_code).await?;
        if !token_response.status().is_success() {
            sleep(Duration::from_secs(1));
            attempts += 1;
        } else {
            break get_response_if_ok::<MicrosoftTokenResponse>(token_response).await?;
        }
    };

    println!("Result {:#?}", microsoft_token_response);
    let xbl_token_response = get_xbl_token(&microsoft_token_response.access_token).await?;

    let xsts_token_response = get_xsts_token(&xbl_token_response.token).await?;
    let user_hash = match xsts_token_response.get_user_hash() {
        Some(user_hash) => user_hash,
        None => return Err(AuthenticationError::XSTSMissingUserHash),
    };

    let minecraft_token_response =
        get_minecraft_token(&xsts_token_response.token, &user_hash).await?;
    // NOTE: Since Xbox Game Pass users don't technically own the game, the entitlement endpoint will show as such.
    // It should be used to check the official public key from liblauncher.so but whats the point in checking if
    // a user owns the game before attempting the next step, if it won't work for Xbox Game Pass users anyway?
    // let _ = check_license(&minecraft_token_response.access_token).await?;

    let mincraft_profile_response =
        get_minecraft_profile(&minecraft_token_response.access_token).await?;

    let account = MinecraftAccount::new(
        mincraft_profile_response,
        microsoft_token_response,
        minecraft_token_response,
    );
    println!("Account: {:#?}", account);
    Ok(account)
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    user_code: String,
    device_code: String,
    verification_uri: String,
    expires_in: u32,
    interval: u32,
    message: String,
}

async fn get_microsoft_devicecode() -> AuthenticationResult<DeviceCodeResponse> {
    let client = Client::new();
    let response = client
        .get(MICROSOFT_DEVICE_CODE_URL)
        .query(&[CLIENT_ID, DEVICE_CODE_SCOPE])
        .send()
        .await?;

    get_response_if_ok::<DeviceCodeResponse>(response).await
}

async fn poll_microsoft_token_endpoint(device_code: &str) -> AuthenticationResult<Response> {
    let mut form: HashMap<&str, &str> = HashMap::new();
    form.insert("device_code", device_code);
    form.insert(DEVICE_CODE_GRANT_TYPE.0, DEVICE_CODE_GRANT_TYPE.1);
    form.insert(CLIENT_ID.0, CLIENT_ID.1);

    let client = Client::new();
    Ok(client.post(MICROSOFT_TOKEN_URL).form(&form).send().await?)
}

async fn get_response_if_ok<'de, T>(response: Response) -> AuthenticationResult<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if status.is_success() {
        Ok(response.json::<T>().await?)
    } else {
        let err_response = response.json::<MicrosoftErrorResponse>().await;
        match err_response {
            Ok(error) => Err(AuthenticationError::from(error)),
            Err(_) => Err(AuthenticationError::HttpResponseError(status)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct XboxTokenResponse {
    #[serde(rename = "IssueInstant")]
    _issue_instant: String,
    #[serde(rename = "NotAfter")]
    _not_after: String,
    #[serde(rename = "Token")]
    token: String,
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
    check_xbox_error(response).await
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

    check_xbox_error(response).await
}

/// Retrieves the successful response from a reqwest::Response from an XBL endpoint
///
/// On error gather any XBL error hints and add them to the XboxError variant
///
/// On a failure response, return the status code of the error.
async fn check_xbox_error(response: reqwest::Response) -> AuthenticationResult<XboxTokenResponse> {
    if response.status().is_success() {
        Ok(response.json::<XboxTokenResponse>().await?)
    } else {
        match response.content_length() {
            Some(content_length) if content_length > 0 => Err(AuthenticationError::from(
                response.json::<XboxErrorResponse>().await?,
            )),
            _ => Err(AuthenticationError::HttpResponseError(response.status())),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MinecraftTokenResponse {
    // This is not the uuid of the mc account
    username: String,
    access_token: String,
    expires_in: u32,
    token_type: String,
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

    if response.status().is_success() {
        let token_response = response.json::<MinecraftTokenResponse>().await?;
        Ok(token_response)
    } else {
        Err(AuthenticationError::HttpResponseError(response.status()))
    }
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

#[derive(Debug, Deserialize)]
pub struct MinecraftProfileSkin {
    id: String,
    state: String,
    url: String,
    variant: String,
    alias: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MinecraftProfileResponse {
    id: String,
    name: String,
    skins: Vec<MinecraftProfileSkin>,
    // TODO: Missing capes, dont know what the response would look like.
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

// Obtains the Minecraft profile information like uuid, username, skins, and capes
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

    if response.status().is_success() {
        Ok(response.json::<MinecraftProfileResponse>().await?)
    } else {
        match response.content_length() {
            Some(content_length) if content_length > 0 => Err(AuthenticationError::from(
                response.json::<MincraftProfileErrorResponse>().await?,
            )),
            _ => Err(AuthenticationError::HttpResponseError(response.status())),
        }
    }
}
