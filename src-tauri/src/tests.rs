// use crate::{
//     purge_old_logs,
//     web_services::{downloader::download_all},
//     MAX_LOGS,
// };

// use chrono::Timelike;
// use serde::{Deserialize, Serialize};
// use std::{
//     env,
//     fs::{self, File},
//     io::BufReader,
//     path::Path,
//     time::Instant,
// };
// use tauri::async_runtime::block_on;

// const TEMP_PATH: &str = ".tmp";

// struct Cleanup;

// impl Drop for Cleanup {
//     fn drop(&mut self) {
//         println!("Cleaning up {}", TEMP_PATH);
//         fs::remove_dir_all(TEMP_PATH).unwrap();
//     }
// }

// #[test]
// fn test_log_purge() {
//     // Cleans up `.tmp` when the function scope ends
//     let _cleanup = Cleanup;
//     // Create a bunch of "log" files and write them to `.tmp`
//     let dir_path = Path::new(TEMP_PATH);
//     fs::create_dir_all(&dir_path).unwrap();
//     let base_datetime = chrono::Local::now();
//     let mut datetimes: Vec<String> = Vec::new();
//     for second in 0..=59 {
//         let datetime = base_datetime.with_second(second).unwrap();
//         let datetime_str = format!(
//             "launcher_log_{}.log",
//             datetime.format("%Y-%m-%dT%H:%M:%S").to_string()
//         );
//         datetimes.push(datetime_str.clone());
//         let path = dir_path.join(&datetime_str);
//         File::create(path).unwrap();
//     }
//     datetimes.reverse();

//     // Purge log files from `.tmp`
//     purge_old_logs(&dir_path).unwrap();

//     // Make sure the resulting files in `.tmp` are the `MAX_LOGS` latest logs.
//     let file_paths = fs::read_dir(dir_path).unwrap();

//     let mut dir_entries = file_paths.filter_map(|path| path.ok()).collect::<Vec<_>>();
//     dir_entries.sort_by_key(|key| key.file_name());
//     dir_entries.reverse();
//     let result: Vec<String> = dir_entries
//         .iter()
//         .map(|entry| entry.file_name().to_str().unwrap().into())
//         .collect();
//     assert!(vec_compare(&result, &datetimes[..MAX_LOGS]));
// }

// #[derive(Debug, Deserialize, Serialize)]
// struct Libraries {
//     libraries: Vec<Library>,
// }

// #[test]
// fn test_downloader() {
//     // Cleans up `.tmp` when the function scope ends
//     let _cleanup = Cleanup;

//     let var = env::var("CARGO_MANIFEST_DIR");
//     assert!(var.is_ok());
//     let path = Path::new(&var.unwrap())
//         .join("resources")
//         .join("libraries_test_1.19.2.json");
//     let file = File::open(path);
//     assert!(file.is_ok());

//     let reader = BufReader::new(file.unwrap());
//     let libs = serde_json::from_reader::<BufReader<File>, Libraries>(reader);
//     assert!(libs.is_ok());

//     let start = Instant::now();
//     let result = block_on(download_all(&libs.unwrap().libraries, Path::new(".tmp")));
//     assert!(result.is_ok());
//     println!(
//         "Successfully downloaded libraries in {}ms",
//         start.elapsed().as_millis()
//     );
// }

// #[test]
// fn test_java_download() {
//     let x = block_on(download_java_version_manifest());
//     println!("{:#?}", x);
//     assert!(x.is_ok())
// }

// #[test]
// fn test_java_runtime_manifest_download() {
//     let url = "https://launchermeta.mojang.com/v1/packages/e968e71afd3360e5032deac19e1c14d7aa32f5bb/manifest.json";
//     let x = block_on(download_java_runtime_manifest(url));
//     println!("{:#?}", x);
//     assert!(x.is_ok())
// }

// fn vec_compare<T>(va: &[T], vb: &[T]) -> bool
// where
//     T: PartialEq,
// {
//     (va.len() == vb.len()) && va.iter().zip(vb).all(|(a, b)| a == b)
// }
