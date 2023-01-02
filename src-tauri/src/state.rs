use log::error;
use tauri::{Manager, Wry};

pub mod account_manager;
pub mod resource_manager;
pub mod instance_manager;

/// Attempts to redirect the main window to the specified endpoint
/// Specify endpoint without a leading `/`.  
pub fn redirect(app_handle: &tauri::AppHandle<Wry>, endpoint: &str) -> tauri::Result<()> {
    let window_name = "main";
    let main_window = app_handle.get_window(&window_name);
    match main_window {
        // If main window exists, try to redirect
        Some(window) => {
            let js = format!(
                "window.location.replace('http://localhost:8080/{}')",
                endpoint
            );
            Ok(window.eval(&js)?)
        }
        // REVIEW: If launcher ever goes to tray, this might need to be changed.
        // If main window doesn't exist then we shouldn't exist, panic.
        None => {
            let error = format!(
                "Trying to access window `{}` but it does not exist anymore.",
                &window_name
            );
            error!("{}", error);
            panic!("{}", error);
        }
    }
}
