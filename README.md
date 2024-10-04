# VibraDB
## What is Vibra?
Vibra is a powerful and fast database that is thread-safe. Vibra takes inspiration from Laravel's Eloquent and SQLite.

Along with its ease-of-use and speed, Vibra is powerfully encrypted using 25 rounds of AES-256 encryption. This ensures that your data is safe and secure.

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
    vibra_db.insert_row("users", row.clone()).await;

    if let Some(value) = vibra_db.get_row("users", "user1").await {
        println!("Retrieved: {:?}", value);
    }

    vibra_db.delete_row("users", "user1").await;
}
```
