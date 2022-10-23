use tauri::{Manager, Wry};
use url::{Url};


const CLIENT_ID: &str = "94fd28d0-faa6-4d85-920d-69a2abe16bcd";
const MICROSOFT_LOGIN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const REDIRECT_URL: &str = "https://login.microsoftonline.com/common/oauth2/nativeclient";

// TODO: Add custom error that is a serializable and deserializable wrapper over a Box<dyn Error>
#[tauri::command]
pub async fn microsoft_login(app_handle: tauri::AppHandle<Wry>) -> Result<(), String> {
    let login_url = Url::parse_with_params(MICROSOFT_LOGIN_URL, &[
        ("prompt", "select_account"),
        ("client_id", CLIENT_ID),
        ("response_type", "code"),
        ("scope", "XboxLive.signin offline_access"),
        ("redirect_url", "https://localhost:8080"),
    ]);

    match login_url {
        Ok(url) => {
            let login_window = tauri::WindowBuilder::new(
                &app_handle,
                "login",
                tauri::WindowUrl::External(url),
            ).build();
            match login_window {
                Ok(window) => {
                    Ok(())
                }
                Err(e) => {
                    Err(e.to_string())
                }
            }
        }
        Err(e) => {
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn ms_login(app_handle: tauri::AppHandle<Wry>) -> Result<(), String> {
    let login_url = Url::parse_with_params(MICROSOFT_LOGIN_URL, &[
        ("prompt", "select_account"),
        ("client_id", CLIENT_ID),
        ("response_type", "code"),
        ("scope", "XboxLive.signin offline_access"),
        ("redirect_url", REDIRECT_URL),
    ]).unwrap();
    let window = app_handle.get_window("login").unwrap();
    window.show();
    window.eval(format!("window.location.replace('{}')", login_url).as_str());



    Ok(())
}