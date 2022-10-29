#![cfg_attr(
    all(not(debug_assertions), target_os = "linux"),
    windows_subsystem = "linux"
)]

use tauri::{http::ResponseBuilder, Manager};

mod web;

use web::{authenticate, show_microsoft_login_page};

fn main() {
    tauri::Builder::default()
        .register_uri_scheme_protocol("autmc", |app_handle, request| {
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
                let res = authenticate(&req).await;
                println!("Auth Result: {:#?}", res);
            });
            let body: Vec<u8> = "<h1>Hello World!</h1>".as_bytes().to_vec();
            ResponseBuilder::new().mimetype("text/html").body(body)
        })
        .invoke_handler(tauri::generate_handler![show_microsoft_login_page])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
