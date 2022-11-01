#![cfg_attr(
    all(not(debug_assertions), target_os = "linux"),
    windows_subsystem = "linux"
)]
mod auth;
mod loader;

use log::{info, warn};
use std::{
    fs::{self},
    path::PathBuf,
};
use tauri::{http::{ResponseBuilder, Request, Response}, App, Manager, Wry, AppHandle};
use serde::ser::{StdError};
use auth::{authenticate, show_microsoft_login_page, validate_active_account, AccountState, AuthMode};
// use loader::obtain_vanilla_manifest;

fn main() {
    tauri::Builder::default()
        .setup(setup)
        .register_uri_scheme_protocol("autmc", autmc_uri_scheme)
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { .. } => {
                info!("Closing");
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![show_microsoft_login_page, validate_active_account])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// First thing called on application setup. 
fn setup(app: &mut App<Wry>) -> Result<(), Box<(dyn StdError + 'static)>> {
    let path_resolver = app.path_resolver();

    let app_dir = path_resolver.app_dir().unwrap();
    fs::create_dir_all(&app_dir)?;

    let log_dir = path_resolver.log_dir().unwrap();
    fs::create_dir_all(&log_dir)?;
    init_logger(&log_dir)?;
    info!("Starting Autmc");

    // Attach the account manager to the app using 'AccountState'
    app.manage(AccountState::new(&app_dir));

    let app_handle = app.handle();
    // Spawn an async thread and use the app_handle to refresh active account.
    tauri::async_runtime::spawn(async move {
        let account_state: tauri::State<AccountState> = app_handle.state();
        let mut account_manager = account_state.0.lock().await;
        match account_manager.deserialize_accounts() {
            Ok(_) => {}
            Err(_) => {
                // If no accounts are saved, bail on this thread since user will need to enter credentials.
                warn!("No account.json exists!");
                return;
            }
        }
        let deserialized_account = account_manager.get_active_account();
        // If there is some active account, retrieve it and attempt to refresh access tokens.
        if let Some(active_account) = deserialized_account {
            let auth_mode =
                AuthMode::MicrosoftRefresh(active_account.microsoft_refresh_token.clone());
            // TODO: Add another function that will figure out which auth mode to use based on account expiry dates, etc...
            let auth_result = authenticate(auth_mode).await;
            match auth_result {
                Ok(account) => {
                    info!("Successfully refreshed account: {}", &account.uuid);
                    account_manager.add_and_activate_account(account);
                }
                Err(err) => {
                    warn!("Error trying to refresh account: {:#?}", err);
                    return;
                }
            }
        }
    });
    Ok(())
}

/// Callback for when a window is redirected to 'autmc://' 
fn autmc_uri_scheme(app_handle: &AppHandle<Wry>, request: &Request) -> Result<Response, Box<dyn std::error::Error>> {
    info!("Retrieved request to custom uri scheme 'autmc://'");
    if let Some(window) = app_handle.get_window("login") {
        // FIXME: Check for window closing errors anyway
        // Neither of the following should be possible in this instance.
        // - Panics if the event loop is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
        // - Panics when called on the main thread, usually on the [`run`](crate::App#method.run) closure.
        window.close().unwrap();
    }
    let url = request.uri().to_owned();

    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        // TODO: Emit an authentication error
        let auth_mode = AuthMode::Full(url);
        let res = authenticate(auth_mode).await;
        println!("Auth Result: {:#?}", &res);
        let account = res.unwrap();

        let account_state: tauri::State<AccountState> = handle.state();
        let mut account_manager = account_state.0.lock().await;

        account_manager.add_and_activate_account(account);
        match account_manager.serialize_accounts() {
            Ok(_) => {}
            Err(err) => warn!("Could not properly serialize account information: {}", err),
        }
    });
    let body: Vec<u8> = "<h1>Hello World!</h1>".as_bytes().to_vec();
    ResponseBuilder::new().mimetype("text/html").body(body)
}

/// Sets up the logger and saves launcher logs to ${app_dir}/logs/launcher_log_${datetime}.log
// TODO: Check historical logs and remove oldest, keeping only latest (20?) logs
fn init_logger(log_dir: &PathBuf) -> Result<(), fern::InitError> {
    let datetime = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
    if !log_dir.is_dir() {
        fs::create_dir(&log_dir)?;
    }
    let log_path = log_dir.join(format!("launcher_log_{}.log", datetime));
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}:{} {}][{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_path.as_os_str())?)
        .apply()?;
    Ok(())
}
