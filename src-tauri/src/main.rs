#![cfg_attr(
all(not(debug_assertions), target_os = "linux"),
windows_subsystem = "linux"
)]

use std::fs::{canonicalize, read};
use tauri::http::ResponseBuilder;
use tauri::Manager;

mod web;

use web::{microsoft_login, ms_login};

fn main() {
    tauri::Builder::default()
        .register_uri_scheme_protocol("tauri", |app, request| {
            let mut path = request.uri().replace("tauri://auth", "");
            println!("path: {}", path);
            let content = read(canonicalize(&path)?)?;
            let (data, meta) = if path.ends_with(".html") {
                (content, "text/html")
            } else if path.ends_with(".js") {
                (content, "text/javascript")
            } else if path.ends_with(".png") {
                (content, "image/png")
            } else {
                unimplemented!();
            };
            ResponseBuilder::new().header("Access-Control-Allow-Origin", "tauri://auth").mimetype(meta).body(data)
        })
        .invoke_handler(tauri::generate_handler![microsoft_login, ms_login])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}