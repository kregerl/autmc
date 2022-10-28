use std::collections::HashMap;
use tauri::{Wry};
use url::{Url};

const CLIENT_ID: &str = "94fd28d0-faa6-4d85-920d-69a2abe16bcd";
const SCOPE: &str = "XboxLive.signin offline_access";
const REDIRECT_URL: &str = "https://login.microsoftonline.com/common/oauth2/nativeclient";
const MICROSOFT_LOGIN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MICROSOFT_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

#[derive(Debug)]
pub enum AuthenticationError {
    UrlParseError(url::ParseError),
    UnknownQueryParameter(String),
    MicrosoftError{error_type: String, error_description: String},
    RequestError(reqwest::Error),
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

// TODO: Add custom error that is a serializable and deserializable wrapper over a Box<dyn Error>
#[tauri::command]
pub async fn show_microsoft_login_page(app_handle: tauri::AppHandle<Wry>) -> Result<(), String> {
    let login_url = Url::parse_with_params(
        MICROSOFT_LOGIN_URL,
        &[
            ("prompt", "select_account"),
            ("client_id", CLIENT_ID),
            ("response_type", "code"),
            ("scope", "XboxLive.signin offline_access"),
            ("redirect_uri", "https://login.microsoftonline.com/common/oauth2/nativeclient"),
        ],
    );

    // Redirects to the custom protocol 'autmc://auth', preserving the query parameters.
    const INIT_SCRIPT: &str = r#"
        if (window.location.href.startsWith("https://login.microsoftonline.com/common/oauth2/nativeclient")) {
            window.location.replace(`autmc://auth${window.location.search}`);
        }
    "#;

    match login_url {
        Ok(url) => {
            let window_url = tauri::WindowUrl::App(url.to_string().parse().unwrap());
            let _login_window = tauri::WindowBuilder::new(&app_handle, "login", window_url)
                .initialization_script(INIT_SCRIPT)
                .build();
            Ok(())
            // match login_window {
            //     Ok(window) => {
            //         Ok(())
            //     }
            //     Err(e) => {
            //         Err(e.to_string())
            //     }
            // }
        }
        Err(e) => Err(e.to_string()),
    }
}

// Fully authenciate with microsoft, xboxlive, and finally minecraft. 
// TODO: Add extra parameters once the flow is worked out to allow refresh tokens to work.
pub async fn authenticate(uri: &str) -> Result<(), AuthenticationError> {
    let parsed_url = Url::parse(uri)?;
    let parameter_map: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    println!("{:#?}", parameter_map);
    if parameter_map.contains_key("code") {
        let authorization_code = parameter_map.get("code").unwrap();
        let client = reqwest::Client::new();
        // Send the post request with the body. 
        let res = client
            .post(MICROSOFT_TOKEN_URL)
            .form(&[
                ("client_id", CLIENT_ID),
                ("scope", SCOPE),
                ("code", authorization_code),
                // redirect_uri must exactly match the returned redirect uri.
                ("redirect_uri", REDIRECT_URL),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await?;

        println!("Url: {:#?}", &res.url());
        println!("{:#?}", res.text().await);

        // TODO: Use code
        Ok(())
    } else if parameter_map.contains_key("error") && parameter_map.contains_key("error_description") {
        // Should not be able to get an error without an error_description
        let error_type = parameter_map.get("error").unwrap();
        let error_description = parameter_map.get("error_description").unwrap();

        Err(AuthenticationError::MicrosoftError{
            error_type: error_type.into(), 
            error_description: error_description.into()
        })
    } else {
        Err(AuthenticationError::UnknownQueryParameter(format!(
            "Unknown query parameters in url {}",
            uri
        )))
    }
}
