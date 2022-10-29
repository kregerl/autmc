use phf::phf_map;
use reqwest::StatusCode;
use serde::{ser::SerializeStructVariant, Deserialize, Serialize};
use serde_json::{json};
use std::{collections::HashMap};
use tauri::Wry;
use url::Url;

const CLIENT_ID: &str = "94fd28d0-faa6-4d85-920d-69a2abe16bcd";
const SCOPE: &str = "XboxLive.signin offline_access";
const REDIRECT_URL: &str = "https://login.microsoftonline.com/common/oauth2/nativeclient";
const MICROSOFT_LOGIN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MICROSOFT_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBOX_LIVE_AUTHENTICATE_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XTXS_AUTHENTICATE_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_AUTHENTICATE_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_LICENSE_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

static XERR_HINTS: phf::Map<&'static str, &'static str> = phf_map! {
    "2148916233" => "2148916233: The account doesn't have an Xbox account. Once they sign up for one (or login through minecraft.net to create one) then they can proceed with the login. This shouldn't happen with accounts that have purchased Minecraft with a Microsoft account, as they would've already gone through that Xbox signup process.",
    "2148916235" => "2148916235: The account is from a country where Xbox Live is not available/banned",
    "2148916236" => "2148916236: The account needs adult verification on Xbox page. (South Korea)",
    "2148916237" => "2148916237: The account needs adult verification on Xbox page. (South Korea)",
    "2148916238" => "The account is a child (under 18) and cannot proceed unless the account is added to a Family by an adult. This only seems to occur when using a custom Microsoft Azure application. When using the Minecraft launchers client id, this doesn't trigger."
};

// REVIEW: Remove '_' prefix from unused fields when they're used. Just there to make the compilier happy. :)
// REVIEW: Many unused fields, serde will ignore unknown fields while deserializing... Remove these?
#[allow(unused)]
#[derive(Debug, Deserialize)]
struct MicrosoftTokenSuccess {
    token_type: String,
    scope: String,
    expires_in: u32,
    // Probably dont need this see https://stackoverflow.com/questions/45681890/oauth-with-azure-ad-v2-0-what-is-the-ext-expires-in-parameter-returned-by-azure
    ext_expires_in: u32,
    access_token: String,
    refresh_token: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MicrosoftTokenResponse {
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

#[derive(Debug, Deserialize)]
struct XboxTokenSuccess {
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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum XboxTokenResponse {
    Success(XboxTokenSuccess),
    // IDEA: Find a way to actually test error responses
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
#[derive(Debug, Deserialize)]
struct MinecraftTokenResponse {
    // This is not the uuid of the mc account
    username: String,
    access_token: String,
    expires_in: u32,
    token_type: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct MinecraftProfileSkin {
    id: String,
    state: String,
    url: String,
    variant: String,
    alias: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct MinecraftProfileSuccess {
    id: String, 
    name: String,
    skins: Vec<MinecraftProfileSkin>,
    // TODO: Missing capes, dont know what the response would look like.
}

#[derive(Debug, Deserialize)]
enum MinecraftProfileResponse {
    Success(MinecraftProfileSuccess),
    Failure {
        #[serde(rename = "errorType")]
        _error_type: String,
        error: String,
        #[serde(rename = "errorMessage")]
        error_message: String,
        #[serde(rename = "developerMessage")]
        _developer_message: String,
    }   
}


#[derive(Debug)]
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
    HttpResponseError(Option<StatusCode>),
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
            AuthenticationError::MinecraftProfileError{
                error, 
                error_message
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
            AuthenticationError::HttpResponseError(msg) => {
                let error = if let Some(message) = msg {
                    format!("400 Bad Request: {}", message)
                } else {
                    "400 Bad Request".into()
                };
                serializer.serialize_str(&error)
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

#[tauri::command]
pub async fn show_microsoft_login_page(app_handle: tauri::AppHandle<Wry>) -> AuthResult<()> {
    let login_url = Url::parse_with_params(
        MICROSOFT_LOGIN_URL,
        &[
            ("prompt", "select_account"),
            ("client_id", CLIENT_ID),
            ("response_type", "code"),
            ("scope", "XboxLive.signin offline_access"),
            (
                "redirect_uri",
                "https://login.microsoftonline.com/common/oauth2/nativeclient",
            ),
        ],
    )?;

    // Redirects to the custom protocol 'autmc://auth', preserving the query parameters.
    const INIT_SCRIPT: &str = r#"
        if (window.location.href.startsWith("https://login.microsoftonline.com/common/oauth2/nativeclient")) {
            window.location.replace(`autmc://auth${window.location.search}`);
        }
    "#;
    let window_url = tauri::WindowUrl::App(login_url.to_string().parse().unwrap());
    // Start window with init script
    let _login_window = tauri::WindowBuilder::new(&app_handle, "login", window_url)
        .initialization_script(INIT_SCRIPT)
        .build()?;
    Ok(())
}

// Fully authenciate with microsoft, xboxlive, and finally minecraft.
// TODO: Add extra parameters once the flow is worked out to allow refresh tokens to work.
pub async fn authenticate(uri: &str) -> AuthResult<()> {
    let microsoft_auth_response = obtain_microsoft_token(uri).await?;
    let xbl_auth_response = obtain_xbl_token(&microsoft_auth_response.access_token).await?;
    println!("Xbl Token: {:#?}", xbl_auth_response);
    println!();
    let xsts_auth_response = obtain_xsts_token(&xbl_auth_response.token).await?;
    println!("Xsts Token: {:#?}", xsts_auth_response);
    println!();
    let user_hash = xsts_auth_response.get_user_hash().unwrap();
    let minecraft_auth_response = obtain_minecraft_token(&xsts_auth_response.token, &user_hash).await?;
    println!("Minecraft Token: {:#?}", minecraft_auth_response);
    println!();
    // REVIEW: Since Xbox Game Pass users don't technically own the game, the entitlement endpoint will show as such.
    // It should be used to check the official public key from liblauncher.so but whats the point in checking if
    // a user owns the game before attempting the next step, if it won't work for Xbox Game Pass users anyway?
    // let _ = check_license(&minecraft_auth_response.access_token).await?;

    let minecraft_profile = obtain_minecraft_profile(&minecraft_auth_response.access_token).await?;
    println!("{:#?}", minecraft_profile);
    Ok(())
}

/// Retrieves the Microsoft access token from the `code` parameter of the redirect uri
async fn obtain_microsoft_token(uri: &str) -> AuthResult<MicrosoftTokenSuccess> {
    // Parse query parameters out of redirect url to get authentication code or errors.
    let parsed_url = Url::parse(uri)?;
    let parameter_map: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    if parameter_map.contains_key("code") {
        let authorization_code = parameter_map.get("code").unwrap();
        let client = reqwest::Client::new();
        // Send the post request with the body.
        let resp = client
            .post(MICROSOFT_TOKEN_URL)
            .form(&[
                ("client_id", CLIENT_ID),
                ("scope", SCOPE),
                ("code", authorization_code),
                // redirect_uri must exactly match the redirect uri from the login url.
                ("redirect_uri", REDIRECT_URL),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await?;

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
            Err(AuthenticationError::HttpResponseError(Some(resp.status())))
        }
    } else if parameter_map.contains_key("error") && parameter_map.contains_key("error_description") {
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

/// Sends request to the XTXS `/authorize` endpoint using an XboxLive access token
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
async fn obtain_minecraft_token(xsts_token: &str, user_hash: &str) -> AuthResult<MinecraftTokenResponse> {
    let client = reqwest::Client::new();
    let response = client
    .post(MINECRAFT_AUTHENTICATE_URL)
    .header("Content-Type", "application/json")
    .header("Accept", "application/json")
    .body(json!({
        "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_token),
        "ensureLegacyEnabled": true
    }).to_string())
    .send()
    .await?;

    if response.status().is_success() {
        let token_response = response.json::<MinecraftTokenResponse>().await?;
        Ok(token_response)
    } else {
        Err(AuthenticationError::HttpResponseError(Some(response.status())))
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

    println!("{:#?}", &response);
    println!("{:#?}", response.text().await?);

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
            MinecraftProfileResponse::Failure { error, error_message, .. } =>  {
                Err(AuthenticationError::MinecraftProfileError {
                    error,
                    error_message,
                })
            },
        }
    } else {
        Err(AuthenticationError::HttpResponseError(Some(response.status())))
    }
}


/// Retrieves the successful response from a reqwest::Response
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
                let hint = *XERR_HINTS.get( &xerr.to_string()).unwrap();
                Err(AuthenticationError::XboxError {
                    xerr: xerr.to_string(),
                    message: message.into(),
                    hint: hint.into()
                })
            }
        }
    } else {
        Err(AuthenticationError::HttpResponseError(Some(response.status())))
    }
}
