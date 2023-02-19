use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};
use tauri::{async_runtime::Mutex as AsyncMutex, AppHandle, Manager, Wry};

use crate::web_services::resources::substitute_account_specific_arguments;

use super::account_manager::Account;

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceConfiguration {
    pub instance_name: String,
    pub jvm_path: PathBuf,
    pub arguments: Vec<String>,
    pub modloader_type: String,
    pub modloader_version: String,
}

pub struct InstanceState(pub Arc<AsyncMutex<InstanceManager>>);

impl InstanceState {
    pub fn new(app_dir: &PathBuf) -> Self {
        Self(Arc::new(AsyncMutex::new(InstanceManager::new(app_dir))))
    }
}

pub struct InstanceManager {
    app_dir: PathBuf,
    instance_map: HashMap<String, InstanceConfiguration>,
    // <Instance name, child process>
    children: HashMap<String, Arc<Mutex<Child>>>,
}

impl InstanceManager {
    pub fn new(app_dir: &Path) -> Self {
        Self {
            app_dir: app_dir.into(),
            instance_map: HashMap::new(),
            children: HashMap::new(),
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
            let instance_path = path.path().join("config.json");
            let file = File::open(&instance_path);
            if let Err(e) = file {
                warn!("Error with instance at {}: {}", instance_path.display(), e);
                continue;
            }
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

    pub fn get_instance_names(&self) -> Vec<String> {
        self.instance_map
            .iter()
            .map(|(instance_name, _)| instance_name.into())
            .collect()
    }

    pub fn launch_instance(&mut self, instance_name: &str, active_account: &Account) {
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
                    .stdout(Stdio::piped());
                debug!("Command: {:#?}", command);
                let child = command.spawn().expect("Could not spawn instance.");
                self.children.insert(instance_name.into(), Arc::new(Mutex::new(child)));
            }
            None => error!("Unknown instance name: {}", instance_name),
        }
    }

    pub fn emit_logs_for_running_instance(&self, app_handle: AppHandle<Wry>) {
        if let Some(instance) = self.get_running_instance() {

            // FIXME: Save thread handle in a map and when and instance is exited, 'join' the thread handle to get its status.
            // https://doc.rust-lang.org/std/thread/
            // To learn when a thread completes, it is necessary to capture the JoinHandle object that is 
            // returned by the call to spawn, which provides a join method that allows the caller to 
            // wait for the completion of the spawned thread:
            thread::spawn(move || {
                if let Ok(mut child) = instance.lock() {
                    let stdout= child.stdout.as_mut().unwrap();
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        match line {
                            Ok(l) => app_handle.emit_all("instance-logging", l).unwrap(),
                            Err(error) => error!("Error reading child process's stdout: {}", error),
                        }
                    }
                } 
            });
        }
    }

    // FIXME: This is just getting a random running instance sine we only really support 1 running instance currently.
    fn get_running_instance(&self) -> Option<Arc<Mutex<Child>>> {
        match self.children.iter().next() {
            Some(entry) => Some(entry.1.clone()),
            None => None,
        }
    }
}
