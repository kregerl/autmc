use log::{error, warn, debug};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{self, BufReader, BufRead, Write},
    path::{Path, PathBuf},
    sync::Arc, fs::{File, self}, process::{Command, Stdio, Child},
};
use tauri::async_runtime::Mutex;

use crate::web_services::resources::substitute_account_specific_arguments;

use super::account_manager::Account;

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceConfiguration {
    pub instance_name: String,
    pub jvm_path: PathBuf,
    pub arguments: Vec<String>,
}

pub struct InstanceState(pub Arc<Mutex<InstanceManager>>);

impl InstanceState {
    pub fn new(app_dir: &PathBuf) -> Self {
        Self(Arc::new(Mutex::new(InstanceManager::new(app_dir))))
    }
}

pub struct InstanceManager {
    app_dir: PathBuf,
    instance_map: HashMap<String, InstanceConfiguration>,
    // <Instance name, child process>
    children: HashMap<String, Child>,
}

impl InstanceManager {
    pub fn new(app_dir: &Path) -> Self {
        Self {
            app_dir: app_dir.into(),
            instance_map: HashMap::new(),
            children: HashMap::new()
        }
    }

    pub fn instances_dir(&self) -> PathBuf {
        self.app_dir.join("instances")
    }

    // FIXME: This is just getting a random running instance sine we only really support 1 running instance currently.
    pub fn get_running_instance(&self) -> Option<&Child> {
        match self.children.iter().next() {
            Some(entry) => Some(entry.1),
            None => None,
        }
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
                self.children.insert(instance_name.into(), child);
                // let reader = BufReader::new(child.stdout.unwrap());
                // for line in reader.lines() {
                //     debug!("Mc Log: {:#?}", line);
                // }
            }
            None => error!("Unknown instance name: {}", instance_name),
        }
    }
}
