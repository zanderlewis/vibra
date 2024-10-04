# VibraDB
## What is Vibra?
Vibra is a powerful, real-time key-value store that is thread-safe. Vibra takes inspiration from Laravel's Eloquent and SQLite.

Along with its ease-of-use and real-time capabilities, Vibra is powerfully encrypted using Kyber. Vibra's Kyber encryption is tripled, ensuring that your data is safe and secure, while still being fast.

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
    };

    // Initialize VibraDB with custom configurations
    let vibra_db = VibraDB::new(config, generate_key(), generate_iv());

    // Example usage
    vibra_db.insert("key1", "value1").await;

    if let Some(value) = vibra_db.get("key1").await {
        println!("Retrieved: {:?}", value);
    }
    
    vibra_db.delete("key1").await;
}
```
