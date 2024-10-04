# VibraDB
[![Rust](https://github.com/zanderlewis/vibra/actions/workflows/rust.yml/badge.svg)](https://github.com/zanderlewis/vibra/actions/workflows/rust.yml)
## What is Vibra?
Vibra is a powerful and fast database that is thread-safe. Vibra takes inspiration from Laravel's Eloquent and SQLite.

Along with its ease-of-use and speed, Vibra is powerfully encrypted using 10 rounds of AES-256 encryption by default. This ensures that your data is safe and secure.

## Installation
Vibra can be added to your `Cargo.toml` file like so:
```toml
[package]
name = "my_vibra_project"
version = "0.0.1"
edition = "2021"

[dependencies]
vibradb = <vibra_version_here>
```

## Vibra.toml
Vibra requires a `Vibra.toml` file to be present in the root of your project. This file contains the configurations for VibraDB. Here is an example of a `Vibra.toml` file:
```toml
path = "vibra_db"
cache_size = 100
enctyption_layers = 10
```

## Usage
```rs
use vibradb::{VibraConfig, VibraDB, Row};
use tokio;
#[tokio::main]
async fn main() {

    // Set up configuration
    let config = match VibraConfig::init() {
        Ok(config) => config,
        Err(e) => {
            println!("Failed to read config file: {}", e);
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
}
```
