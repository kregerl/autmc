use std::collections::HashMap;

use log::debug;
use reqwest::{StatusCode, Url};
use serde::{ser::SerializeStructVariant, Deserialize, Serialize};
use serde_json::json;

use crate::state::account_manager::Account;

use super::consts::{
    CLIENT_ID, MICROSOFT_TOKEN_URL, MINECRAFT_AUTHENTICATE_URL, MINECRAFT_LICENSE_URL,
    MINECRAFT_PROFILE_URL, REDIRECT_URL, SCOPE, XBOX_LIVE_AUTHENTICATE_URL, XERR_HINTS,
    XTXS_AUTHENTICATE_URL,
};

// REVIEW: Remove '_' prefix from unused fields when they're used. Just there to make the compilier happy. :)
// REVIEW: Many unused fields, serde will ignore unknown fields while deserializing... Remove these and the #[allow(unused)]?
#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
pub struct MicrosoftTokenSuccess {
    token_type: String,
    scope: String,
    expires_in: u32,
    // Probably dont need this see https://stackoverflow.com/questions/45681890/oauth-with-azure-ad-v2-0-what-is-the-ext-expires-in-parameter-returned-by-azure
    ext_expires_in: u32,
    access_token: String,
    refresh_token: String,
}

#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MicrosoftTokenResponse {
    Success(MicrosoftTokenSuccess),
    Failure {
        error: String,
        error_description: String,
        error_codes: Vec<u32>,
        timestamp: String,
        trace_id: String,
        correlation_id: String,
        // Redirect is used for consoles.
        error_uri: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XboxTokenSuccess {
    #[serde(rename = "IssueInstant")]
    _issue_instant: String,
    #[serde(rename = "NotAfter")]
    _not_after: String,
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: HashMap<String, Vec<HashMap<String, String>>>,
}

impl XboxTokenSuccess {
    pub fn get_user_hash(&self) -> Option<String> {
        let xui = self.display_claims.get("xui")?;
        let uhs = xui.first()?.get("uhs")?;
        Some(uhs.into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum XboxTokenResponse {
    Success(XboxTokenSuccess),
    // TODO: Find a way to actually test error responses
    Failure {
        #[serde(rename = "Identity")]
        _identity: String,
        #[serde(rename = "XErr")]
        xerr: u32,
        #[serde(rename = "Message")]
        message: String,
        // Redirect is used for consoles.
        #[serde(rename = "Redirect")]
        _redirect: String,
    },
}

#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftTokenResponse {
    // This is not the uuid of the mc account
    username: String,
    pub access_token: String,
    pub expires_in: u32,
    token_type: String,
}

#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
struct MinecraftProfileSkin {
    id: String,
    state: String,
    url: String,
    variant: String,
    alias: String,
}

#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftProfileSuccess {
    id: String,
    name: String,
    skins: Vec<MinecraftProfileSkin>,
    // TODO: Missing capes, dont know what the response would look like.
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MinecraftProfileResponse {
    Success(MinecraftProfileSuccess),
    Failure {
        #[serde(rename = "errorType")]
        _error_type: String,
        error: String,
        #[serde(rename = "errorMessage")]
        error_message: String,
        #[serde(rename = "developerMessage")]
        _developer_message: String,
    },
}

#[allow(unused)]
pub enum AuthMode {
    /// Contains the redirect uri
    Full(String),
    /// Contains the Microsoft refresh token
    MicrosoftRefresh(String),
    /// Contains the Microsoft access token, Microsoft refresh token, and access token expiry
    MinecraftRefresh {
        access_token: String,
        refresh_token: String,
        access_token_expiry: i64,
    },
}

enum MicrosoftGrantType {
    /// Contains the authorization code
    Authorization(String),
    /// Contains the refresh token
    Refresh(String),
}

#[derive(Debug)]
// TODO: Implement Display for this error type.
pub enum AuthenticationError {
    MicrosoftError {
        error_type: String,
        error_description: String,
    },
    XboxError {
        xerr: String,
        message: String,
        hint: String,
    },
    MinecraftProfileError {
        error: String,
        error_message: String,
    },
    UnknownQueryParameter(String),
    UrlParseError(url::ParseError),
    RequestError(reqwest::Error),
    WindowError(tauri::Error),
    HttpResponseError(StatusCode),
}

pub type AuthResult<T> = core::result::Result<T, AuthenticationError>;

impl Serialize for AuthenticationError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Ok(match self {
            AuthenticationError::MicrosoftError {
                error_type,
                error_description,
            } => {
                let mut state = serializer.serialize_struct_variant(
                    "AuthenticationError",
                    0,
                    "MicrosoftError",
                    2,
                )?;
                state.serialize_field("error_type", &error_type)?;
                state.serialize_field("error_description", &error_description)?;
                state.end()
            }
            AuthenticationError::XboxError {
                xerr,
                message,
                hint,
            } => {
                let mut state = serializer.serialize_struct_variant(
                    "AuthenticationError",
                    1,
                    "XboxError",
                    3,
                )?;
                state.serialize_field("xerr", &xerr)?;
                state.serialize_field("message", &message)?;
                state.serialize_field("hint", &hint)?;
                state.end()
            }
            AuthenticationError::MinecraftProfileError {
                error,
                error_message,
            } => {
                let mut state = serializer.serialize_struct_variant(
                    "AuthenticationError",
                    2,
                    "MinecraftProfileError",
                    2,
                )?;
                state.serialize_field("error", &error)?;
                state.serialize_field("error_message", &error_message)?;
                state.end()
            }
            AuthenticationError::UnknownQueryParameter(error) => serializer.serialize_str(&error),
            AuthenticationError::UrlParseError(error) => {
                serializer.serialize_str(&error.to_string())
            }
            AuthenticationError::RequestError(error) => {
                serializer.serialize_str(&error.to_string())
            }
            AuthenticationError::WindowError(error) => serializer.serialize_str(&error.to_string()),
            AuthenticationError::HttpResponseError(status_code) => {
                serializer.serialize_str(&format!("Status code: {}", status_code))
            }
        }?)
    }
}

impl From<url::ParseError> for AuthenticationError {
    fn from(e: url::ParseError) -> Self {
        AuthenticationError::UrlParseError(e)
    }
}

impl From<reqwest::Error> for AuthenticationError {
    fn from(e: reqwest::Error) -> Self {
        AuthenticationError::RequestError(e)
    }
}

impl From<tauri::Error> for AuthenticationError {
    fn from(e: tauri::Error) -> Self {
        AuthenticationError::WindowError(e)
    }
}

/// Fully authenciate with microsoft, xboxlive, and finally minecraft.
pub async fn authenticate(auth_mode: AuthMode) -> AuthResult<Account> {
    // Timestamp in seconds
    let now = chrono::Local::now().timestamp();
    // Depending on the auth mode, will request necessary tokens.
    let microsoft_token = match auth_mode {
        AuthMode::Full(uri) => {
            // If were doing a full auth flow, retrieve the code from the redirect uri.
            let authorization_code = retrieve_authorization_code(&uri)?;
            let auth_mode = MicrosoftGrantType::Authorization(authorization_code);
            let microsoft_auth_response = obtain_microsoft_token(auth_mode).await?;
            let expiry = now + (microsoft_auth_response.expires_in - 10) as i64;
            (
                microsoft_auth_response.access_token,
                microsoft_auth_response.refresh_token,
                expiry,
            )
        }
        AuthMode::MicrosoftRefresh(refresh_token) => {
            // Use the refresh token to get a new access token.
            let microsoft_auth_response =
                obtain_microsoft_token(MicrosoftGrantType::Refresh(refresh_token)).await?;
            let expiry = now + (microsoft_auth_response.expires_in - 10) as i64;
            (
                microsoft_auth_response.access_token,
                microsoft_auth_response.refresh_token,
                expiry,
            )
        }
        // Return the microsoft access token since it is still valid
        AuthMode::MinecraftRefresh {
            access_token,
            refresh_token,
            access_token_expiry,
        } => (access_token, refresh_token, access_token_expiry),
    };
    debug!("Microsoft: {:#?}", microsoft_token);
    let xbl_auth_response = obtain_xbl_token(&microsoft_token.0).await?;
    debug!("Xbl Token: {:#?}", xbl_auth_response);
    let xsts_auth_response = obtain_xsts_token(&xbl_auth_response.token).await?;
    debug!("Xsts Token: {:#?}", xsts_auth_response);
    let user_hash = xsts_auth_response.get_user_hash().unwrap();
    let minecraft_auth_response =
        obtain_minecraft_token(&xsts_auth_response.token, &user_hash).await?;
    let minecraft_auth_expiry = now + (minecraft_auth_response.expires_in - 10) as i64;
    debug!("Minecraft Token: {:#?}", minecraft_auth_response);
    // REVIEW: Since Xbox Game Pass users don't technically own the game, the entitlement endpoint will show as such.
    // It should be used to check the official public key from liblauncher.so but whats the point in checking if
    // a user owns the game before attempting the next step, if it won't work for Xbox Game Pass users anyway?
    // let _ = check_license(&minecraft_auth_response.access_token).await?;

    let minecraft_profile = obtain_minecraft_profile(&minecraft_auth_response.access_token).await?;
    debug!("minecraft_profile {:#?}", minecraft_profile);
    Ok(Account {
        uuid: minecraft_profile.id,
        name: minecraft_profile.name,
        // IDEA: Skin url for head?
        microsoft_access_token: microsoft_token.0,
        microsoft_access_token_expiry: microsoft_token.2,
        microsoft_refresh_token: microsoft_token.1,
        minecraft_access_token: minecraft_auth_response.access_token,
        minecraft_access_token_expiry: minecraft_auth_expiry,
    })
}

pub async fn validate_account(account: &Account) -> AuthResult<Account> {
    let now = chrono::Local::now().timestamp();
    // Account expired.
    if account.minecraft_access_token_expiry <= now {
        debug!(
            "Minecraft Token Expired. Now: {} Expiry: {}",
            now, account.minecraft_access_token_expiry
        );
        if account.microsoft_access_token_expiry <= now {
            debug!("Microsoft Token Expired.");
            // Refresh access token.
            let auth_mode = AuthMode::MicrosoftRefresh(account.microsoft_refresh_token.to_owned());
            let auth_result = authenticate(auth_mode).await?;
            Ok(auth_result)
        } else {
            debug!("Microsoft token Valid.");
            // MS access_token is fine, use that for minecraft auth.
            let auth_mode = AuthMode::MinecraftRefresh {
                access_token: account.microsoft_access_token.to_owned(),
                refresh_token: account.microsoft_refresh_token.to_owned(),
                access_token_expiry: account.microsoft_access_token_expiry,
            };
            let auth_result = authenticate(auth_mode).await?;
            Ok(auth_result)
        }
    } else {
        debug!("Minecraft Token Valid.");
        // Account has not expired
        Ok(account.clone())
    }
}

/// Parse query parameters out of redirect url to get authentication code or errors.
fn retrieve_authorization_code(uri: &str) -> AuthResult<String> {
    let parsed_url = Url::parse(uri)?;
    let parameter_map: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    if parameter_map.contains_key("code") {
        let authorization_code = parameter_map.get("code").unwrap();
        Ok(authorization_code.into())
    } else if parameter_map.contains_key("error") && parameter_map.contains_key("error_description")
    {
        // Should not be able to get an error without an error_description
        let error_type = parameter_map.get("error").unwrap();
        let error_description = parameter_map.get("error_description").unwrap();

        Err(AuthenticationError::MicrosoftError {
            error_type: error_type.into(),
            error_description: error_description.into(),
        })
    } else {
        Err(AuthenticationError::UnknownQueryParameter(format!(
            "Unknown query parameters in url {}",
            uri
        )))
    }
}

/// Retrieves the Microsoft access token from the `code` parameter of the redirect uri
async fn obtain_microsoft_token(
    auth_mode: MicrosoftGrantType,
) -> AuthResult<MicrosoftTokenSuccess> {
    let mut form: HashMap<&str, &str> = HashMap::new();
    form.insert("client_id", CLIENT_ID);
    form.insert("scope", SCOPE);
    form.insert("redirect_uri", REDIRECT_URL);

    let code = match auth_mode {
        MicrosoftGrantType::Authorization(authorization_code) => {
            form.insert("grant_type", "authorization_code");
            ("code", authorization_code)
        }
        MicrosoftGrantType::Refresh(refresh_token) => {
            form.insert("grant_type", "refresh_token");
            ("refresh_token", refresh_token)
        }
    };
    form.insert(code.0, &code.1);

    let client = reqwest::Client::new();
    // Send the post request with the body.
    let resp = client.post(MICROSOFT_TOKEN_URL).form(&form).send().await?;

    // If request was successful, retrieve the token or error
    if resp.status().is_success() {
        let token_response = resp.json::<MicrosoftTokenResponse>().await?;
        match token_response {
            MicrosoftTokenResponse::Success(success) => Ok(success),
            MicrosoftTokenResponse::Failure {
                error,
                error_description,
                ..
            } => Err(AuthenticationError::MicrosoftError {
                error_type: error,
                error_description,
            }),
        }
    } else {
        Err(AuthenticationError::HttpResponseError(resp.status()))
    }
}

/// Sends request to the XboxLive `/authenticate` endpoint using a Microsoft access token
async fn obtain_xbl_token(access_token: &str) -> AuthResult<XboxTokenSuccess> {
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
async fn obtain_xsts_token(xbl_token: &str) -> AuthResult<XboxTokenSuccess> {
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

/// Sends request to the mojang `/login_with_xbox` endpoint using the user hash and XSTS token
async fn obtain_minecraft_token(
    xsts_token: &str,
    user_hash: &str,
) -> AuthResult<MinecraftTokenResponse> {
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

#[allow(unused)]
/// Unused for now, currently cannot show if a Xbox Game Pass user owns the game so whats the point in checking...
async fn check_license(access_token: &str) -> AuthResult<()> {
    let client = reqwest::Client::new();
    let response = client
        .get(MINECRAFT_LICENSE_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;

    debug!("{:#?}", &response);
    debug!("{:#?}", response.text().await?);

    Ok(())
}

// Obtains the Minecraft profile information like uuid, username, skins, and capes
async fn obtain_minecraft_profile(access_token: &str) -> AuthResult<MinecraftProfileSuccess> {
    let client = reqwest::Client::new();
    let response = client
        .get(MINECRAFT_PROFILE_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;

    if response.status().is_success() {
        let profile_response = response.json::<MinecraftProfileResponse>().await?;
        match profile_response {
            MinecraftProfileResponse::Success(success) => Ok(success),
            MinecraftProfileResponse::Failure {
                error,
                error_message,
                ..
            } => Err(AuthenticationError::MinecraftProfileError {
                error,
                error_message,
            }),
        }
    } else {
        Err(AuthenticationError::HttpResponseError(response.status()))
    }
}

/// Retrieves the successful response from a reqwest::Response from an XBL endpoint
///
/// On error gather any XBL error hints and add them to the XboxError variant
///
/// On a failure response, return the status code of the error.
async fn check_xbox_error(response: reqwest::Response) -> AuthResult<XboxTokenSuccess> {
    if response.status().is_success() {
        let token_response = response.json::<XboxTokenResponse>().await?;
        match token_response {
            XboxTokenResponse::Success(success) => Ok(success),
            XboxTokenResponse::Failure { xerr, message, .. } => {
                let hint = *XERR_HINTS.get(&xerr.to_string()).unwrap();
                Err(AuthenticationError::XboxError {
                    xerr: xerr.to_string(),
                    message: message.into(),
                    hint: hint.into(),
                })
            }
        }
    } else {
        Err(AuthenticationError::HttpResponseError(response.status()))
    }
}
