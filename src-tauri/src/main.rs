#![cfg_attr(
    all(not(debug_assertions), target_os = "linux"),
    windows_subsystem = "windows"
)]

mod commands;
mod consts;
mod state;
#[cfg(test)]
mod tests;
mod web_services;

use commands::show_microsoft_login_page;
use log::{error, info, warn, debug};
use regex::Regex;
use serde::ser::StdError;
use state::{account_manager::AccountState, redirect};
use std::{
    fs::{self},
    path::{Path, PathBuf},
};
use tauri::{
    http::{Request, Response, ResponseBuilder},
    App, AppHandle, Manager, Wry,
};
use web_services::authentication::{authenticate, validate_account, AuthMode};

use crate::{
    commands::{
        get_account_skin, get_instance_path, launch_instance, load_instances, obtain_manifests,
        obtain_version,
    },
    state::{instance_manager::InstanceState, resource_manager::ResourceState},
};

const MAX_LOGS: usize = 20;
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            Ok(match setup(app) {
                Ok(_) => println!("Here"),
                Err(e) => debug!("Error: {:#?}", e),
            })
        })
        .register_uri_scheme_protocol("autmc", autmc_uri_scheme)
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { .. } => {
                info!("Closing");
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            show_microsoft_login_page,
            obtain_manifests,
            obtain_version,
            get_instance_path,
            load_instances,
            get_account_skin,
            launch_instance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// First thing called on application setup.
fn setup(app: &mut App<Wry>) -> Result<(), Box<(dyn StdError + 'static)>> {
    let path_resolver = app.path_resolver();

    let app_dir = path_resolver.app_config_dir().unwrap();
    fs::create_dir_all(&app_dir)?;

    let log_dir = path_resolver.app_log_dir().unwrap();
    fs::create_dir_all(&log_dir)?;
    match init_logger(&log_dir) {
        Ok(x) => {},
        Err(e) => println!("Error: {}", e),
    }
    info!("Starting Autmc");

    // Attach the account manager to the app using 'AccountState'
    app.manage(AccountState::new(&app_dir));
    app.manage(ResourceState::new(&app_dir));
    app.manage(InstanceState::new(&app_dir));
    let app_handle = app.handle();

    // Spawn an async thread and use the app_handle to refresh active account.
    // TODO: Maybe emit event to display a toast telling the user what happened.
    tauri::async_runtime::spawn(async move {
        // Download manifests into the resource manager
        let resource_state: tauri::State<ResourceState> = app_handle
            .try_state()
            .expect("`ResourceState` should already be managed.");
        let mut resource_manager = resource_state.0.lock().await;
        match resource_manager.download_manifests().await {
            Ok(_) => {}
            Err(error) => error!("Manifest Error: {:#?}", error),
        }

        let account_state: tauri::State<AccountState> = app_handle
            .try_state()
            .expect("`AccountState` should already be managed.");
        let mut account_manager = account_state.0.lock().await;
        match account_manager.deserialize_accounts() {
            Ok(_) => {}
            Err(_) => {
                // If no accounts are saved, bail on this thread since user will need to enter credentials.
                info!("No account.json exists!");

                if let Err(error) = redirect(&app_handle, "login") {
                    error!("{}", error.to_string());
                }
                return;
            }
        }
        let deserialized_account = account_manager.get_active_account();
        // If there is some active account, retrieve it and attempt to refresh access tokens.
        match deserialized_account {
            Some(active_account) => {
                let validation_result = validate_account(active_account).await;
                // FIXME: Dont just unwrap, give user any auth errors.
                let account = validation_result.unwrap();
                // Save account to account manager.
                account_manager.add_and_activate_account(account);

                match account_manager.serialize_accounts() {
                    Ok(_) => {}
                    Err(err) => warn!("Could not properly serialize account information: {}", err),
                }
            }
            None => {
                if let Err(error) = redirect(&app_handle, "login") {
                    error!("{}", error.to_string());
                }
            }
        }
    });
    Ok(())
}

/// Callback for when a window is redirected to 'autmc://'
fn autmc_uri_scheme(
    app_handle: &AppHandle<Wry>,
    request: &Request,
) -> Result<Response, Box<dyn std::error::Error>> {
    info!("Retrieved request to custom uri scheme 'autmc://'");
    if let Some(window) = app_handle.get_window("login") {
        // Neither of the following should be possible in this instance.
        // - Panics if the event loop is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
        // - Panics when called on the main thread, usually on the [`run`](crate::App#method.run) closure.
        window.close().unwrap();
    }
    let url = request.uri().to_owned();
    let handle = app_handle.clone();
    // Spawn a thread to handle authentication.
    tauri::async_runtime::spawn(async move {
        // TODO: Emit an authentication error
        let auth_mode = AuthMode::Full(url);
        let res = authenticate(auth_mode).await;
        println!("Auth Result: {:#?}", &res);
        // FIXME: Dont just unwrap, give user any auth errors.
        let account = res.unwrap();

        let account_state: tauri::State<AccountState> = handle
            .try_state()
            .expect("`AccountState` should already be managed.");
        let mut account_manager = account_state.0.lock().await;

        // Save account to account manager.
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
fn init_logger(log_dir: &PathBuf) -> Result<(), fern::InitError> {
    let datetime = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
    if !log_dir.is_dir() {
        fs::create_dir(&log_dir)?;
    }
    println!("Before purge");
    purge_old_logs(&log_dir)?;
    println!("After purge");
    //let log_path = log_dir.join(format!("launcher_log_{}.log", datetime));
    let log_path = log_dir.join("launcher_log.log");
    println!("Log path: {:#?}", log_path);
    let latest_log_path = log_dir.join("latest.log");
    if latest_log_path.exists() {
        fs::remove_file(&latest_log_path)?;
    }
    println!("After latest.log");
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
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_path.as_os_str()).expect("Error here"))
        .chain(fern::log_file(latest_log_path.as_os_str())?)
        .apply()?;
    Ok(())
}

/// Removes `old` logs, keeping only the latest MAX_LOGS in the log directory.
fn purge_old_logs(log_dir: &Path) -> Result<(), std::io::Error> {
    let file_paths = fs::read_dir(log_dir)?;
    println!("{:#?}", file_paths);

    let regex = Regex::new("^launcher_log_[0-9]{4}-[0-9]{2}-[0-9]{2}T([0-9]{2}:){2}[0-9]{2}.log$");
    match regex {
        Ok(rexp) => {
            let mut dir_entries = file_paths
                .filter_map(|path| path.ok())
                .filter_map(|entry| {
                    if rexp.is_match(entry.file_name().to_str().unwrap()) {
                        Some(entry)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            dir_entries.sort_by_key(|key| key.file_name());
            dir_entries.reverse();
            if dir_entries.len() > MAX_LOGS {
                let removable_entries = &dir_entries[MAX_LOGS..];
                for entry in removable_entries {
                    fs::remove_file(log_dir.join(entry.file_name()))?;
                }
            }
        }
        Err(err) => warn!("{}", err),
    }
    Ok(())
}
