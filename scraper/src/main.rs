mod databases;
mod webclient;
mod config;

use crate::databases::mongodb::MongoHandler;
use crate::webclient::webclient::WebClient;
use std::thread;
use crate::config::config::Config;

#[tokio::main]
async fn main() {

    let config = Config::new();

    let mongodb_handler = match MongoHandler::new(&config.mongodb_uri, &config.mongodb_db_name).await {
        Ok(handler) => {
            println!("Successfully connected to MongoDB");
            handler
        },
        Err(e) => {
            eprintln!("Failed to connect to MongoDB: {}", e);
            return;
        }
    };

    mongodb_handler.initiate_collections().await;

    mongodb_handler.initiate_configs(&config).await;

    let web_client = WebClient::new();

    loop {
        thread::sleep(std::time::Duration::from_secs(config.check_interval));

        let activated_bosses = mongodb_handler.get_activated_bosses().await;
        for boss_config in activated_bosses {
            let response = web_client.getpulls(&boss_config, &config).await;
            match response {
                Ok(resp) => {
                    if &resp.status() != &200 {
                        eprintln!("Error fetching pulls for boss {} with difficulty {}: HTTP {}", boss_config.boss_id, boss_config.difficulty_id, resp.status());
                        break;
                    }
                    
                    println!("Successfully fetched pulls for boss {} with difficulty {}", boss_config.boss_id, boss_config.difficulty_id);
                    let pulls_data = resp.json::<serde_json::Value>().await.unwrap();

                    let formatted_pulls = pulls_data["bossAttempts"].as_array().unwrap().iter().map(|pull| {
                        let mut pull_doc = mongodb::bson::Document::new();
                        pull_doc.insert("pull_id", pull.get("id").and_then(|v| v.as_i64()).unwrap_or(0));
                        
                        // Get difficulty from raid data
                        let difficulty = pull.get("raid").and_then(|v| v.get("difficulty")).and_then(|v| v.as_str()).unwrap_or("");
                        
                        // Store encounter data as nested document
                        let mut encounter_doc = mongodb::bson::Document::new();
                        if let Some(encounter) = pull.get("encounter") {
                            let boss_slug = encounter.get("slug").and_then(|v| v.as_str()).unwrap_or("");
                            encounter_doc.insert("slug", boss_slug);
                            encounter_doc.insert("difficulty", difficulty);
                            
                            // Store boss data
                            let mut boss_doc = mongodb::bson::Document::new();
                            boss_doc.insert("slug", boss_slug);
                            encounter_doc.insert("boss", boss_doc);
                            
                            // Store raid data
                            let mut raid_doc = mongodb::bson::Document::new();
                            if let Some(raid) = pull.get("raid") {
                                raid_doc.insert("slug", raid.get("slug").and_then(|v| v.as_str()).unwrap_or(""));
                            }
                            encounter_doc.insert("raid", raid_doc);
                        }
                        pull_doc.insert("encounter", encounter_doc);
                        
                        // Store realm data
                        let mut realm_doc = mongodb::bson::Document::new();
                        realm_doc.insert("slug", config.realm_slug.clone());
                        pull_doc.insert("realm", realm_doc);
                        
                        // Store guild data
                        let mut guild_doc = mongodb::bson::Document::new();
                        guild_doc.insert("slug", config.guild_slug.clone());
                        pull_doc.insert("guild", guild_doc);

                        let is_pull_autoreject: &str = match pull.get("duration_ms").and_then(|v| v.as_i64()).unwrap_or(0) < 10000 {
                            true => "rejected",
                            false => "waiting"
                        };
                        
                        pull_doc.insert("is_success", pull.get("is_success").and_then(|v| v.as_bool()).unwrap_or(false));
                        pull_doc.insert("overall_percent", pull.get("overall_percent").and_then(|v| v.as_f64()).unwrap_or(0.0));
                        pull_doc.insert("pull_count", pull.get("pull_count").and_then(|v| v.as_i64()).unwrap_or(0));
                        pull_doc.insert("duration_ms", pull.get("duration_ms").and_then(|v| v.as_i64()).unwrap_or(0));
                        pull_doc.insert("status", is_pull_autoreject);

                        pull_doc
                    }).collect::<Vec<mongodb::bson::Document>>();
                    mongodb_handler.write_pulls(formatted_pulls).await;
                },
                Err(e) => {
                    eprintln!("Failed to fetch pulls for boss {} with difficulty {}: {}", boss_config.boss_id, boss_config.difficulty_id, e);
                }
            }
        }
    }
}