mod config;
mod db;
mod models;

use crate::config::VibraConfig;
use crate::db::VibraDB;
use crate::models::Row;
use tokio;
use log::{info, error};

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Set up configuration
    let config = match VibraConfig::from_file("Vibra.toml") {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to read config file: {}", e);
            return;
        }
    };

    // Initialize VibraDB with custom configurations
    let vibra_db = VibraDB::new(config);

    // Example usage
    vibra_db.create_table("users").await;

    let row = Row {
        id: "user1".to_string(),
        columns: vec![
            ("name".to_string(), "John Doe".to_string()),
            ("email".to_string(), "john.doe@example.com".to_string()),
        ],
    };

    vibra_db.insert_row("users", row).await;

    if let Some(value) = vibra_db.get_row("users", "user1").await {
        println!("Retrieved: {:?}", value);
    } else {
        println!("Failed to retrieve row");
    }

    let updated_row = Row {
        id: "user1".to_string(),
        columns: vec![
            ("name".to_string(), "John Doe Updated".to_string()),
            ("email".to_string(), "john.doe.updated@example.com".to_string()),
        ],
    };

    vibra_db.update_row("users", updated_row).await;
    
    if let Some(value) = vibra_db.get_row("users", "user1").await {
        println!("Retrieved: {:?}", value);
    } else {
        println!("Failed to retrieve row");
    }

    // Delete DB
    vibra_db.delete_db().await;

    info!("All operations completed successfully");
}