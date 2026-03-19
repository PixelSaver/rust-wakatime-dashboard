use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use serde_json;
use dirs;

use crate::auth::TokenResponse;

fn token_cache_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().ok_or(anyhow::anyhow!("Failed to find user config directory."))?;
    path.push(".wakatime_dash");
    fs::create_dir_all(&path)?;
    path.push("token.json");
    Ok(path)
}

pub fn save_token(token: &TokenResponse) -> Result<()> {
    let path = token_cache_path()?;
    fs::write(path, serde_json::to_string_pretty(token)?)?;
    Ok(())
}
pub fn load_token() -> Option<TokenResponse> {
    let path = token_cache_path().ok()?;
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}