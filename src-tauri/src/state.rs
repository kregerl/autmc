use std::sync::Arc;

use futures::Future;
use log::{debug, error};
use tauri::{Manager, Wry};
use tokio::sync::{Mutex, OwnedMutexGuard};

pub mod account_manager;
pub mod instance_manager;
pub mod resource_manager;

/// Attempts to redirect the main window to the specified endpoint
/// Specify endpoint without a leading `/`.  
pub fn redirect(app_handle: &tauri::AppHandle<Wry>, endpoint: &str) -> tauri::Result<()> {
    let window_name = "main";
    let main_window = app_handle.get_window(window_name);
    debug!("Redirecting {} window to /{}", window_name, endpoint);
    match main_window {
        // If main window exists, try to redirect
        Some(window) => {
            let mut new_url = window.url();
            new_url.set_path(endpoint);

            let js = format!(
                "window.location.replace('{}')",
                new_url
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

pub trait InnerState<T> {
    fn inner_state(&self) -> T;
}

pub trait ManagerFromAppHandle {
    type State;

    fn from_app_handle(app_handle: &tauri::AppHandle) -> impl Future<Output = OwnedMutexGuard<Self>>
    where
        <Self as ManagerFromAppHandle>::State: 'static + InnerState<Arc<Mutex<Self>>> + Send + Sync,
    {
        let state: tauri::State<Self::State> = app_handle
            .try_state()
            .expect("This state should already be managed.");
        state.inner_state().lock_owned()
    }
}
