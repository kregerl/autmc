use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};
use tauri::{
    async_runtime::{JoinHandle, Mutex as AsyncMutex},
    AppHandle, Manager, Wry,
};
use tokio::io::{AsyncBufReadExt, BufReader as AsyncBufReader};
use tokio::process::{Child, Command};

use crate::web_services::resources::{substitute_account_specific_arguments, ModloaderType};

use super::account_manager::Account;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InstanceConfiguration {
    pub instance_name: String,
    pub jvm_path: PathBuf,
    pub arguments: Vec<String>,
    pub modloader_type: ModloaderType,
    pub modloader_version: String,
}

pub struct InstanceState(pub Arc<AsyncMutex<InstanceManager>>);

impl InstanceState {
    pub fn new(app_dir: &Path) -> Self {
        Self(Arc::new(AsyncMutex::new(InstanceManager::new(app_dir))))
    }
}

pub struct InstanceManager {
    app_dir: PathBuf,
    instance_map: HashMap<String, InstanceConfiguration>,
    // <Instance name, child process>
    children: HashMap<String, Arc<AsyncMutex<Child>>>,
    logging_threads: HashMap<String, JoinHandle<()>>,
}

impl InstanceManager {
    pub fn new(app_dir: &Path) -> Self {
        Self {
            app_dir: app_dir.into(),
            instance_map: HashMap::new(),
            children: HashMap::new(),
            logging_threads: HashMap::new(),
        }
    }

    pub fn instances_dir(&self) -> PathBuf {
        self.app_dir.join("instances")
    }

    /// Add the config.json to an instance folder. Used to relaunch the instance again.
    pub fn add_instance(&self, config: InstanceConfiguration) -> Result<(), io::Error> {
        let path = self
            .instances_dir()
            .join(&config.instance_name)
            .join("config.json");
        let mut file = File::create(path)?;
        let json = serde_json::to_string(&config)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn deserialize_instances(&mut self) {
        let paths = fs::read_dir(self.instances_dir());
        if let Err(e) = paths {
            error!("Error loading instances from disk: {}", e);
            return;
        }
        for path in paths.unwrap().filter_map(|path| path.ok()) {
            // Skip over non-directory files
            let path_meta = path.metadata();
            if let Ok(metadata) = path_meta {
                if !metadata.is_dir() {
                    continue;
                }
            }
            // Append "config.json" to the dir path
            let instance_path = path.path().join("config.json");
            let file = File::open(&instance_path);
            if let Err(e) = file {
                warn!("Error with instance at {}: {}", instance_path.display(), e);
                continue;
            }
            // Deserialize the instance configuration
            let reader = BufReader::new(file.unwrap());
            let instance =
                serde_json::from_reader::<BufReader<File>, InstanceConfiguration>(reader);
            if let Err(e) = instance {
                warn!(
                    "Error loading `config.json` for instance at {}: {}",
                    instance_path.display(),
                    e
                );
                continue;
            }
            let conf = instance.unwrap();
            self.instance_map.insert(conf.instance_name.clone(), conf);
        }
    }

    pub fn get_instance_configurations(&self) -> Vec<InstanceConfiguration> {
        self.instance_map.values().map(|instance| instance.clone()).collect()
    }

    pub fn get_instance_names(&self) -> Vec<String> {
        self.instance_map
            .keys()
            .map(|instance_name| instance_name.into())
            .collect()
    }

    pub fn launch_instance(
        &mut self,
        instance_name: &str,
        active_account: &Account,
        app_handle: AppHandle<Wry>,
    ) {
        debug!("Instance Name: {}", instance_name);
        let instance_config = self.instance_map.get(instance_name);
        match instance_config {
            Some(instance) => {
                let working_dir = self.instances_dir().join(instance_name);
                let mut args: Vec<String> = Vec::new();
                for argument in &instance.arguments {
                    args.push(
                        match substitute_account_specific_arguments(argument, active_account) {
                            Some(arg) => arg,
                            None => argument.into(),
                        },
                    );
                }
                let mut command = Command::new(&instance.jvm_path);
                command
                    .current_dir(working_dir)
                    .args(args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                debug!("Command: {:#?}", command);
                let child = command.spawn().expect("Could not spawn instance.");

                let child_handle = Arc::new(AsyncMutex::new(child));
                self.tick_instance(instance_name.into(), child_handle.clone(), app_handle);
                self.children.insert(instance_name.into(), child_handle);
            }
            None => error!("Unknown instance name: {}", instance_name),
        }
    }

    fn tick_instance(
        &mut self,
        instance_name: String,
        child_handle: Arc<AsyncMutex<Child>>,
        app_handle: AppHandle<Wry>,
    ) {
        let name = instance_name.clone();
        let handle = tauri::async_runtime::spawn(async move {
            let mut child = child_handle.lock().await;
            let stdout = child
                .stdout
                .take()
                .expect("Child did not have stdout handle.");
            let stderr = child
                .stderr
                .take()
                .expect("Child did not have stderr handle.");

            let mut stdout_reader = AsyncBufReader::new(stdout).lines();
            let mut stderr_reader = AsyncBufReader::new(stderr).lines();

            #[derive(Serialize, Clone)]
            struct Logging {
                instance_name: String,
                category: String,
                line: String,
            }

            // TODO: Emit an event to the screenshot store when a screenshot is taken. use notifier crate.
            loop {
                tokio::select! {
                    result = stdout_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => {
                                app_handle.emit_all("instance-logging", Logging { instance_name: instance_name.clone(), category: "running".into(), line }).unwrap();
                            },
                            Err(_) => break,
                            _ => (),
                        }
                    }
                    result = stderr_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => debug!("Emit stderr line: {}", line),
                            Err(_) => break,
                            _ => (),
                        }
                    }
                    result = child.wait() => {
                        match result {
                            Ok(exit_status) => {
                                debug!("Child exited with exit code: {}", exit_status);
                                break;
                            },
                            Err(_) => break,
                        }
                    }
                };
            }
        });
        self.logging_threads.insert(name, handle);
    }
}
