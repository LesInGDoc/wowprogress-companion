use mongodb::{ Client };
use crate::config::config::Config;
use crate::webclient::webclient::BossConfig;
use serde_json::Value;

pub struct MongoHandler {
    address: String,
    client: Client,
    database: Option<mongodb::Database>,
}

impl MongoHandler {
    pub async fn new(address: &str, db_name: &str) -> Result<MongoHandler, mongodb::error::Error> {
        let client = Client::with_uri_str(address).await?;

        let database = client.database(db_name);

        Ok(MongoHandler {
            address: address.to_string(),
            client,
            database: Some(database),
        })
    }

    pub async fn initiate_collections(&self) {
        if let Some(database) = &self.database {

            database.collection::<mongodb::bson::Document>("configs");

            database.collection::<mongodb::bson::Document>("raids");

            database.collection::<mongodb::bson::Document>("pulls");

        }
    }

    pub async fn initiate_configs(&self, config: &Config) {
        if let Some(database) = &self.database {

            let configs_collection = database.collection::<mongodb::bson::Document>("configs");

            let empty_filter = mongodb::bson::doc! {};
            if configs_collection.count_documents(empty_filter).await.unwrap() == 0 {

                let mut all_bosses = Vec::new();
                for boss_id in config.bosses_ids.iter() {
                    for difficulty_id in config.difficulty_ids.iter() {
                        let boss_config = mongodb::bson::doc! {
                            "boss_id": boss_id.clone(),
                            "activated": false,
                            "difficulty_id": difficulty_id.clone(),
                            "raid_id": config.raid_id.clone()
                        };
                        all_bosses.push(mongodb::bson::Bson::Document(boss_config));
                    }
                }

                let raid_config = mongodb::bson::doc! {
                    "type": "raid",
                    "raid_id": config.raid_id.clone(),
                    "bosses_ids": config.bosses_ids.clone(),
                    "difficulty_ids": config.difficulty_ids.clone(),
                    "bosses_config": all_bosses,
                };

                configs_collection.insert_one(raid_config).await.unwrap();
            }
        }
    }

    pub async fn get_activated_bosses(&self) -> Vec<BossConfig> {
        let mut activated_bosses = Vec::new();

        if let Some(database) = &self.database {
            let configs_collection = database.collection::<mongodb::bson::Document>("configs");

            let filter = mongodb::bson::doc! { "type": "raid" };
            if let Ok(Some(config_doc)) = configs_collection.find_one(filter).await {
                if let Some(bosses_config) = config_doc.get_array("bosses_config").ok() {
                    for boss_config in bosses_config {
                        if let Some(boss_doc) = boss_config.as_document() {
                            if let Some(activated) = boss_doc.get_bool("activated").ok() {
                                if activated {
                                    let boss_id = boss_doc.get_str("boss_id").unwrap_or("").to_string();
                                    let difficulty_id = boss_doc.get_str("difficulty_id").unwrap_or("").to_string();
                                    let raid_id = boss_doc.get_str("raid_id").unwrap_or("").to_string();

                                    activated_bosses.push(BossConfig {
                                        boss_id,
                                        activated,
                                        difficulty_id,
                                        raid_id,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        activated_bosses
    }

    pub async fn get_pull_by_id(&self, pull_id: &i64) -> Option<Value> {
        if let Some(database) = &self.database {
            let pulls_collection = database.collection::<mongodb::bson::Document>("pulls");

            let filter = mongodb::bson::doc! { "pull_id": pull_id };
            if let Ok(Some(pull_doc)) = pulls_collection.find_one(filter).await {
                let pull_json = serde_json::to_string(&pull_doc).unwrap();
                return serde_json::from_str(&pull_json).ok();
            }
        }

        None
    }

    pub async fn write_pulls(&self, pulls: Vec<mongodb::bson::Document>) {
        if let Some(database) = &self.database {
            let pulls_collection = database.collection::<mongodb::bson::Document>("pulls");

            if !pulls.is_empty() {
                for pull in pulls.iter() {
                    let pull_id = pull.get_i64("pull_id").unwrap_or(0);

                    if self.get_pull_by_id(&pull_id).await.is_none() {
                        println!("Inserting new pull with id {}", pull_id);
                        pulls_collection.insert_one(pull).await.unwrap();
                    } else {
                        println!("Pull with id {} already exists, skipping insertion", pull_id);
                    }
                }
            }
        }
    }
}