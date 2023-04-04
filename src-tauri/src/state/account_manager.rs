use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Error, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::Mutex, AppHandle, Wry, Manager};
use tokio::time::sleep;

use crate::web_services::authentication::{authenticate, AuthMode};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Account {
    pub uuid: String,
    pub name: String,
    // FIXME: Cache downloaded skins instead of saving url to download everytime.
    pub skin_url: String,
    pub microsoft_access_token: String,
    pub microsoft_access_token_expiry: u64,
    pub microsoft_refresh_token: String,
    pub minecraft_access_token: String,
    pub minecraft_access_token_expiry: u64,
}

#[derive(Debug)]
pub struct AccountState(pub Arc<Mutex<AccountManager>>);

impl AccountState {
    pub fn new(app_dir: &Path) -> Self {
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
        let file = File::open(path)?;
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

    /// Get the active account's uuid
    pub fn get_active_uuid(&self) -> Option<String> {
        self.active.clone()
    }

    /// Return the hashmap of uuid -> account
    pub fn get_all_accounts(&self) -> HashMap<String, Account> {
        self.accounts.clone()
    }

    /// Add and activate an account, overwriting any existing accounts with the same uuid.
    pub fn add_and_activate_account(&mut self, account: Account, app_handle: AppHandle<Wry>) {
        let uuid = &account.uuid.clone();
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        debug!(
            "{} :: Account will need to be refreshed at MC:{} MS:{}",
            since_the_epoch.as_secs(),
            account.minecraft_access_token_expiry,
            account.microsoft_access_token_expiry
        );
        self.add_account(account);
        self.activate_account(uuid, app_handle);
        info!(
            "Added and activated account: {}",
            self.active.as_ref().unwrap()
        );
    }

    // Activate the account associated with uuid
    pub fn activate_account(&mut self, uuid: &str, app_handle: AppHandle<Wry>) {
        self.active = Some(uuid.to_owned());
        // Can unwrap here since we just set `self.active`
        let account = self.get_account(uuid).unwrap().clone();
        // Spawn a thread to refresh access tokens once they expire.
        tauri::async_runtime::spawn(async move {
            // Refresh the tokens 5s earlier than needed. 
            // Assumes SystemTime is after UNIX_EPOCH
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 10;
            let auth_mode =
                if account.minecraft_access_token_expiry < account.microsoft_access_token_expiry {
                    // Minecraft
                    let secs_until_expire = account.minecraft_access_token_expiry - now;
                    sleep(Duration::from_secs(secs_until_expire)).await;
                    info!("Refreshing minecraft access token");
                    AuthMode::MinecraftRefresh {
                        access_token: account.microsoft_access_token,
                        refresh_token: account.microsoft_refresh_token,
                        access_token_expiry: account.microsoft_access_token_expiry,
                    }
                } else {
                    // Microsoft
                    let secs_until_expire = account.microsoft_access_token_expiry - now;
                    sleep(Duration::from_secs(secs_until_expire)).await;
                    info!("Refreshing Microsoft access token");
                    AuthMode::MicrosoftRefresh(account.microsoft_refresh_token)
                };
            let account_state: tauri::State<AccountState> = app_handle
                .try_state()
                .expect("`AccountState` should already be managed.");
            let mut account_manager = account_state.0.lock().await;

            let account_res = authenticate(auth_mode).await;
            match account_res {
                Ok(account) => account_manager.add_and_activate_account(account, app_handle.clone()),
                Err(e) => error!("Issue re-authenticating with microsoft: {}", e.to_string()),
            }
        });
    }

    /// Adds an account, overwriting any existing accounts with the same uuid.
    pub fn add_account(&mut self, account: Account) {
        self.accounts.insert(account.uuid.clone(), account);
    }
}
