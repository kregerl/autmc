use autmc_authentication::{
    refresh_access_tokens, AuthenticationResult, MinecraftAccount, OAuthRefreshMode,
};
use autmc_log::debug_if;
use log::debug;

pub async fn validate_account(account: MinecraftAccount) -> AuthenticationResult<MinecraftAccount> {
    let now = chrono::Local::now().timestamp() as u64;
    let refresh_mode = if account.microsoft_access_token_expiry <= now {
        debug_if!(
            "AUTHENTICATION",
            "Microsoft token expired on {} its now {}",
            account.microsoft_access_token_expiry,
            now,
        );
        Some(OAuthRefreshMode::Microsoft {
            refresh_token: account.microsoft_refresh_token.clone(),
        })
    } else if account.minecraft_access_token_expiry <= now {
        debug_if!(
            "AUTHENTICATION",
            "Microsoft token expired on {} its now {}",
            account.minecraft_access_token_expiry,
            now,
        );
        Some(OAuthRefreshMode::Minecraft {
            token: account.clone().into(),
        })
    } else {
        None
    };

    if let Some(mode) = refresh_mode {
        refresh_access_tokens(mode).await
    } else {
        debug!("Minecraft Token Valid.");
        Ok(account)
    }
}
