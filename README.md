# VibraDB
## What is Vibra?
Vibra is a powerful, real-time key-value store that is thread-safe. Vibra takes inspiration from Laravel's Eloquent and SQLite.

Along with its ease-of-use and real-time capabilities, Vibra is powerfully encrypted using a LAQ-Fort. For more information on encryption, visit the [LAQ-Fort Repository.](https://github.com/zanderlewis/laq-fort)

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

## Usage
```rs
#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Set up configuration
    let config = VibraConfig {
        path: String::from("vibra_db"),
        cache_size: 100,
        encryption_enabled: true,
        aes_layers: 0,
    };

    // Initialize VibraDB with custom configurations
    let vibra_db = VibraDB::new(config, generate_key(), generate_iv());

    // Example usage
    vibra_db.insert("key1", "value1").await;

    if let Some(value) = vibra_db.get("key1").await {
        println!("Retrieved: {:?}", value);
    }
    if let Some(value) = vibra_db.get("kee1").await {
        println!("Retrieved: {:?}", value);
    }

    vibra_db.delete("key1").await;

    vibra_db.insert("key1", "value1").await;
    vibra_db.insert("key2", "value2").await;
    vibra_db.insert("key6", "value6").await;
    vibra_db.insert("kee1", "value0").await;
    vibra_db.insert("key", "value-1").await;
    
    // Range query
    let range_results = vibra_db.range_query("key1", "key5").await;
    println!("{:?}", range_results);
    for (key, value) in range_results {
        println!("Range Result: {} = {}", key, value);
    };

    vibra_db.delete("key6").await;

    // Pattern match query
    let pattern_results = vibra_db.pattern_match(r"key\d").await;
    for (key, value) in pattern_results {
        println!("Pattern Result: {} = {}", key, value);
    };

    vibra_db.delete("key1").await;
    vibra_db.delete("key2").await;
    vibra_db.delete("kee1").await;
    vibra_db.delete("key").await;
}
```
