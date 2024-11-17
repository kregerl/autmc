#![cfg_attr(
    all(not(debug_assertions), target_os = "linux"),
    windows_subsystem = "windows"
)]

mod authentication;
mod commands;
mod consts;
mod option_parser;
mod state;
#[cfg(test)]
mod tests;
mod web_services;
use crate::state::ManagerFromAppHandle;
use crate::{
    authentication::validate_account,
    commands::{
        get_account_skin, get_accounts, get_curseforge_categories, get_logs, get_screenshots,
        import_zip, launch_instance, load_instances, obtain_manifests, obtain_version, open_folder,
        poll_device_code_authentication, read_log_lines, search_curseforge,
        start_authentication_flow,
    },
    state::{
        account_manager::AccountManager, instance_manager::InstanceState,
        resource_manager::ResourceState,
    },
};
use autmc_authentication::AuthenticationError::{MicrosoftError, XboxError};
use log::{error, info, warn};
use regex::Regex;
use serde::ser::StdError;
use state::{account_manager::AccountState, redirect};
use std::{
    fs::{self},
    path::{Path, PathBuf},
};
use tauri::{api::cli::Matches, App, Manager, Wry};

const MAX_LOGS: usize = 20;
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            match setup(app) {
                Ok(_) => {}
                Err(e) => println!("Error: {:#?}", e),
            };
            Ok(())
        })
        // .register_uri_scheme_protocol("autmc", autmc_uri_scheme)
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event.event() {
                info!("Closing");
            }
        })
        .invoke_handler(tauri::generate_handler![
            start_authentication_flow,
            poll_device_code_authentication,
            obtain_manifests,
            obtain_version,
            load_instances,
            get_account_skin,
            launch_instance,
            get_accounts,
            open_folder,
            get_screenshots,
            get_logs,
            read_log_lines,
            import_zip,
            search_curseforge,
            get_curseforge_categories,
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
        Ok(_) => {}
        Err(e) => println!("Error: {}", e),
    }
    info!("Starting Autmc");

    // Attach the account manager to the app using 'AccountState'
    app.manage(AccountState::new(&app_dir));
    app.manage(ResourceState::new(&app_dir));
    app.manage(InstanceState::new(&app_dir));
    let app_handle = app.handle();

    let cli_matches = match app.get_cli_matches() {
        Ok(matches) => matches,
        Err(e) => {
            error!("Invalid CLI Arguments: {}", e);
            app_handle.exit(1);
            Matches::default()
        }
    };

    info!("Arguments: {:#?}", cli_matches);
    // let arguments = cli_matches.args.get("instance").unwrap();
    // if let Value::String(value) = &arguments.value {
    //     launch_instance(value.into(), app_handle.clone()).await;
    // }

    // Spawn an async thread and use the app_handle to refresh active account.
    // TODO: Maybe emit event to display a toast telling the user what happened.
    tauri::async_runtime::spawn(async move {
        let mut account_manager = AccountManager::from_app_handle(&app_handle).await;

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
                let validation_result = validate_account(active_account.clone()).await;

                // If the result if an error, emit error to user
                if let Err(validation_error) = &validation_result {
                    if let Err(error) = app_handle.emit_to(
                        "main",
                        "authentication-error",
                        validation_error.to_string(),
                    ) {
                        error!("{}", error.to_string());
                        return;
                    }
                }

                match validation_result {
                    Ok(account) => {
                        // Save account to account manager.
                        account_manager.add_and_activate_account(account, app_handle.clone());

                        if let Err(error) = account_manager.serialize_accounts() {
                            warn!(
                                "Could not properly serialize account information: {}",
                                error
                            );
                        }
                    },
                    Err(e) => match e {
                        MicrosoftError { .. } | XboxError { .. } => {
                            if let Err(error) = redirect(&app_handle, "login") {
                                error!("{}", error.to_string());
                            }
                        }
                        _ => error!("{}", e.to_string()),
                    },
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

/// Sets up the logger and saves launcher logs to ${app_dir}/logs/launcher_log_${datetime}.log
fn init_logger(log_dir: &PathBuf) -> Result<(), fern::InitError> {
    let datetime = chrono::Local::now().format("%Y-%m-%dT%H-%M-%S");
    if !log_dir.is_dir() {
        fs::create_dir(log_dir)?;
    }
    purge_old_logs(log_dir)?;
    let log_path = log_dir.join(format!("launcher_log_{}.log", datetime));
    // let log_path = log_dir.join("launcher_log.log");
    println!("Log path: {:#?}", log_path);
    let latest_log_path = log_dir.join("latest.log");
    if latest_log_path.exists() {
        fs::remove_file(&latest_log_path)?;
    }
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
        .level(match std::env::var("DEBUG") {
            Ok(var) if var == "1" => log::LevelFilter::Debug,
            _ => log::LevelFilter::Info,
        })
        .level_for(
            "reqwest",
            match std::env::var("REQWEST_DEBUG") {
                Ok(var) if var == "1" => log::LevelFilter::Debug,
                _ => log::LevelFilter::Info,
            },
        )
        .chain(std::io::stdout())
        .chain(fern::log_file(log_path.as_os_str())?)
        .chain(fern::log_file(latest_log_path.as_os_str())?)
        .apply()?;
    Ok(())
}

/// Removes `old` logs, keeping only the latest MAX_LOGS in the log directory.
fn purge_old_logs(log_dir: &Path) -> Result<(), std::io::Error> {
    let file_paths = fs::read_dir(log_dir)?;
    println!("{:#?}", file_paths);

    let regex = Regex::new("^launcher_log_[0-9]{4}-[0-9]{2}-[0-9]{2}T([0-9]{2}-){2}[0-9]{2}.log$");
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
