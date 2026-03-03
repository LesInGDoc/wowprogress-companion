pub struct Config {
    pub mongodb_uri: String,
    pub mongodb_db_name: String,
    pub check_interval: u64,
    pub raid_id: String,
    pub bosses_ids: Vec<String>,
    pub difficulty_ids: Vec<String>,
    pub realm_slug: String,
    pub guild_slug: String,
    pub region_slug: String,
}

impl Config {
    pub fn new() -> Config {
        let mongodb_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let mongodb_db_name = std::env::var("MONGODB_DB_NAME").unwrap_or_else(|_| "wowprogress".to_string());
        let check_interval = std::env::var("CHECK_INTERVAL").unwrap_or_else(|_| "3600".to_string()).parse::<u64>().expect("CHECK_INTERVAL must be a valid integer");
        let raid_id = std::env::var("RAID_ID").unwrap_or_else(|_| "sanctum-of-domination".to_string());
        let bosses_ids = std::env::var("BOSSES_IDS").unwrap_or_else(|_| "the-tatarus,kyrian-threshold,leiden,guardian-of-the-first-ones,remnant-of-narglatch,balakar-kalu'ak".to_string()).split(',').map(|s| s.trim().to_string()).collect();
        let difficulty_ids = std::env::var("DIFFICULTY_IDS").unwrap_or_else(|_| "mythic".to_string()).split(',').map(|s| s.trim().to_string()).collect();
        let realm_slug = std::env::var("REALM_SLUG").unwrap_or_else(|_| "ysondre".to_string());
        let guild_slug = std::env::var("GUILD_SLUG").unwrap_or_else(|_| "WoW%20H%C3%B4tel".to_string());
        let region_slug = std::env::var("REGION_SLUG").unwrap_or_else(|_| "eu".to_string());

        Config {
            mongodb_uri,
            mongodb_db_name,
            check_interval,
            raid_id,
            bosses_ids,
            difficulty_ids,
            realm_slug,
            guild_slug,
            region_slug,
        }
    }
}