use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Error, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use log::info;
use serde::{Deserialize, Serialize};
use tauri::async_runtime::Mutex;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Account {
    pub uuid: String,
    pub name: String,
    // IDEA: Skin url for head?
    pub microsoft_access_token: String,
    pub microsoft_access_token_expiry: i64,
    pub microsoft_refresh_token: String,
    pub minecraft_access_token: String,
    pub minecraft_access_token_expiry: i64,
}

#[derive(Debug)]
pub struct AccountState(pub Arc<Mutex<AccountManager>>);

impl AccountState {
    pub fn new(app_dir: &PathBuf) -> Self {
        Self(Arc::new(Mutex::new(AccountManager::new(app_dir))))
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AccountManager {
    #[serde(skip)]
    path: PathBuf,
    active: Option<String>,
    accounts: HashMap<String, Account>,
}

// FIXME: Storing tokens in plaintext is bad... store them in the platform keystore using keyring-rs
//        Only need to store ms_access_token, ms_refresh_token, and mc_access_token. Everything else can be in a different format.
impl AccountManager {
    /// Call on app setup.
    pub fn new(app_dir: &Path) -> Self {
        Self {
            path: app_dir.into(),
            active: Default::default(),
            accounts: Default::default(),
        }
    }

    /// Deserialize account information into `app_dir/accounts.json`
    pub fn deserialize_accounts(&mut self) -> Result<(), Error> {
        let path = &self.path.join("accounts.json");
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let deserialized_account_manager =
            serde_json::from_reader::<BufReader<File>, AccountManager>(reader)?;
        self.active = deserialized_account_manager.active;
        self.accounts = deserialized_account_manager.accounts;
        Ok(())
    }

    /// Serialize account information into `app_dir/accounts.json`
    pub fn serialize_accounts(&self) -> Result<(), Error> {
        let json = serde_json::to_string(&self)?;
        let path = &self.path.join("accounts.json");
        let mut file = File::create(path)?;
        info!("Serialized account manager.");
        file.write_all(json.as_bytes())
    }

    /// Get a stored account by uuid.
    pub fn get_account(&self, uuid: &str) -> Option<&Account> {
        self.accounts.get(uuid)
    }

    /// Get the active account
    pub fn get_active_account(&self) -> Option<&Account> {
        if let Some(active_uuid) = &self.active {
            self.get_account(active_uuid)
        } else {
            None
        }
    }

    /// Add and activate an account, overwriting any existing accounts with the same uuid.
    pub fn add_and_activate_account(&mut self, account: Account) {
        self.active = Some(account.uuid.clone());
        self.add_account(account);
        info!(
            "Added and activated account: {}",
            self.active.as_ref().unwrap()
        );
    }

    /// Adds an account, overwriting any existing accounts with the same uuid.
    pub fn add_account(&mut self, account: Account) {
        self.accounts.insert(account.uuid.clone(), account);
    }
}
