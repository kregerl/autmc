#![cfg_attr(
    all(not(debug_assertions), target_os = "linux"),
    windows_subsystem = "linux"
)]
mod auth;
mod loader;
mod resource_manager;

use std::{fs, path::PathBuf};
use log::info;
use tauri::{http::ResponseBuilder, Manager};

use auth::{authenticate, show_microsoft_login_page};
use loader::{obtain_vanilla_manifest};

use crate::auth::AuthMode;

fn main() {
    tauri::Builder::default()
        .register_uri_scheme_protocol("autmc", |app_handle, request| {
            info!("Retrieved request to custom uri scheme 'autmc://'");
            println!("{:#?}", request);
            if let Some(window) = app_handle.get_window("login") {
                // FIXME: Check for window closing errors anyway
                // Neither of the following should be possible in this instance. 
                // - Panics if the event loop is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
                // - Panics when called on the main thread, usually on the [`run`](crate::App#method.run) closure.
                window.close().unwrap();
            }
            let req = request.uri().to_owned();
            tauri::async_runtime::spawn(async move {
                // TODO: Emit an authentication error
                let auth_mode = AuthMode::Full(req);
                let res = authenticate(auth_mode).await;
                println!("Auth Result: {:#?}", res);
            });
            let body: Vec<u8> = "<h1>Hello World!</h1>".as_bytes().to_vec();
            ResponseBuilder::new().mimetype("text/html").body(body)
        })
        .setup(|app| {
            let resolver = app.path_resolver();
            let app_dir = app.path_resolver().app_dir();
            match app_dir {
                Some(path) => fs::create_dir_all(path)?,
                None => todo!("Error creating app_dir. TODO: Make user choose app dir."), // TODO: Ask user to set app directory / install path.
            }
            let log_dir = resolver.log_dir().unwrap();
            init_logger(&log_dir)?;
            info!("Starting Tauri App");
            // TODO: Get the manifest after user login.
            // tauri::async_runtime::spawn(async move {
                // let _manifest = obtain_vanilla_manifest().await;
            // });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![show_microsoft_login_page])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Sets up the logger and saves launcher logs to ${app_dir}/logs/launcher_log_${datetime}.log
// TODO: Check historical logs and remove oldest, keeping only latest (20?) logs 
fn init_logger(log_dir: &PathBuf) -> Result<(), fern::InitError> {
    let datetime = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
    fs::create_dir(&log_dir)?;
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