# VibraDB
## What is Vibra?
Vibra is a powerful, real-time key-value store that is thread-safe. Vibra takes inspiration from Laravel's Eloquent and SQLite.

Along with its ease-of-use and real-time capabilities, Vibra is powerfully encrypted using [LAQ-Fort](https://github.com/zanderlewis/laq-fort) with a fractal depth of 250 and a AES multiplier of 250 (by default), meaning you have 500 layers of standard encryption. LAQ-Fort has built-in triple Kyber encryption, along with the custom depth fractal encryption and custom multiplier of AES, ensuring your data is extra secure while keeping the speed of encryption and decryption.

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
        encryption_enabled: false,
    };

    // Initialize VibraDB with custom configurations
    let vibra_db = VibraDB::new(config, generate_aes_key());

    // Example usage
    vibra_db.insert("key1", "value1").await;

    if let Some(value) = vibra_db.get("key1").await {
        println!("Retrieved: {:?}", value);
    }

    vibra_db.delete("key1").await;

    vibra_db.insert("key1", "value1").await;
    vibra_db.insert("key2", "value2").await;


    // Range query
    let range_results = vibra_db.range_query("key1", "key5").await;
    println!("{:?}", range_results);
    for (key, value) in range_results {
        println!("Range Result: {} = {}", key, value);
    };
    // Pattern match query
    let pattern_results = vibra_db.pattern_match(r"key\d").await;
    for (key, value) in pattern_results {
        println!("Pattern Result: {} = {}", key, value);
    };

    vibra_db.delete("key1").await;
    vibra_db.delete("key2").await;
}
```
