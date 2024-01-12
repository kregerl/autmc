use autmc_authentication::{
    refresh_access_tokens, AuthenticationResult, MinecraftAccount, OAuthRefreshMode,
};
use log::debug;

pub async fn validate_account(account: MinecraftAccount) -> AuthenticationResult<MinecraftAccount> {
    let now = chrono::Local::now().timestamp() as u64;
    if account.minecraft_access_token_expiry <= now {
        debug!(
            "Minecraft Token Expired. Now: {} Expiry: {}",
            now, account.minecraft_access_token_expiry
        );
        let auth_mode = if account.microsoft_access_token_expiry <= now {
            debug!("Microsoft Token Expired.");
            // Refresh access token.
            OAuthRefreshMode::Minecraft {
                token: account.into(),
            }
        } else {
            debug!("Microsoft token Valid.");
            // MS access_token is fine, use that for minecraft auth.
            OAuthRefreshMode::Microsoft {
                refresh_token: account.microsoft_refresh_token,
            }
        };
        refresh_access_tokens(auth_mode).await
    } else {
        debug!("Minecraft Token Valid.");
        // Account has not expired
        Ok(account)
    }
}
