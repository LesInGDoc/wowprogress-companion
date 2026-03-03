use reqwest::{Client, Response};
use crate::config::config::Config;

pub struct WebClient {
    pub client: Client
}

pub struct BossConfig {
    pub boss_id: String,
    pub activated: bool,
    pub difficulty_id: String,
    pub raid_id: String,
}

impl WebClient {
    pub fn new() -> WebClient {
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        WebClient { client }
    }

    pub async fn get(&self, url: &str) -> Result<Response, reqwest::Error> {
        self.client.get(url).send().await
    }

    pub async fn getpulls(&self, boss_config: &BossConfig, config: &Config) -> Result<Response, reqwest::Error> {
        let url = format!("https://raider.io/api/guilds/boss-attempts?raid={}&boss={}&difficulty={}&region={}&realm={}&guild={}&limit=999", boss_config.raid_id, boss_config.boss_id, boss_config.difficulty_id, config.region_slug, config.realm_slug, config.guild_slug);
        self.client.get(&url).send().await
    }
}