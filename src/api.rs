use anyhow::anyhow;

use crate::auth::TokenResponse;
use crate::{AppData, Screen, WakatimeDash};

#[derive(Debug, serde::Deserialize)]
pub struct TrustFactor {
    pub trust_level: String,
    pub trust_value: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct Me {
    pub id: u64,
    pub emails: Vec<String>,
    pub slack_id: Option<String>,
    pub github_username: Option<String>,
    // pub trust_factor: TrustFactor,
}

impl WakatimeDash {
    pub fn fetch_me(&mut self) -> anyhow::Result<Me> {
        self.data.is_loading = true;
        let client = reqwest::blocking::Client::new();
        let token = &self
            .data
            .token
            .as_ref()
            .ok_or(anyhow::anyhow!("Token not found"))?
            .access_token;
        // println!("{}", token);

        let url = format!("https://hackatime.hackclub.com/api/v1/authenticated/me");

        let response = client.get(&url).header("Authorization", format!("Bearer {}", token)).send()?;
        
        let status = response.status();
        
        if !status.is_success() {
            let text = response.text()?;
            return Err(anyhow!(
                "Request failed: {} - {}",
                status,
                text,
            ));
        }
        let body = response.json::<Me>()?;
        println!("Me: {:?}", body);
        Ok(body)
    }
}
