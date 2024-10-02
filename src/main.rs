use sled::Db;
use tokio;
use std::sync::{Arc, Mutex};
use log::info;
use lru::LruCache;
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::Rng;
use regex::Regex;
use std::str;
use base64;
use std::fs;

// Constants for AES encryption (key and block sizes)
const AES_KEY_SIZE: usize = 32; // 256 bits

// Configurations for the database
#[derive(Clone)]
struct VibraConfig {
    path: String,
    cache_size: usize,
    encryption_enabled: bool,
    aes_layers: usize,
}

#[derive(Clone)]
struct VibraDB {
    db: Arc<Db>, 
    cache: Arc<Mutex<LruCache<String, String>>>, 
    key: Vec<u8>,
    iv: [u8; 16],
    config: VibraConfig,
}

// Generate a random 256-bit AES key
pub fn generate_key() -> Vec<u8> {
    rand::thread_rng().gen::<[u8; AES_KEY_SIZE]>().to_vec()
}

// Generate a random IV key
pub fn generate_iv() -> [u8; 16] {
    rand::thread_rng().gen::<[u8; 16]>()
}

impl VibraDB {
    // Create a new instance of VibraDB with custom configurations
    pub fn new(config: VibraConfig, key: Vec<u8>, iv: [u8; 16]) -> VibraDB {
        let db = sled::open(&config.path).expect("Failed to open VibraDB");
        info!("VibraDB initialized at {}", config.path);

        let cache = LruCache::new(config.cache_size);

        let lpath = config.path.clone() + "/";
        let rpath = ".gitignore".to_string();
        let path = lpath + &rpath;
        // Create a .gitignore file to automatically ignore the db.
        let f = fs::write(path, b"*\n");
        drop(f);

        VibraDB {
            db: Arc::new(db),
            cache: Arc::new(Mutex::new(cache)),
            key,
            iv,
            config,
        }
    }

    // Encrypt a value
    fn encrypt_value(&self, value: &str) -> Vec<u8> {
        let mut encrypted = value.as_bytes().to_vec();
        let iv = self.iv.clone();
        let key = self.key.clone();

        for _ in 0..self.config.aes_layers {
            let cipher = Cbc::<Aes256, Pkcs7>::new_from_slices(&key, &iv).unwrap();
            encrypted = cipher.encrypt_vec(&encrypted);
        }
        encrypted
    }

    // Decrypt a value
    fn decrypt_value(&self, encrypted: &[u8]) -> String {
        let iv = self.iv.clone();
        let mut decrypted = encrypted.to_vec();
        let key = self.key.clone();

        for _ in 0..self.config.aes_layers {
            let cipher = Cbc::<Aes256, Pkcs7>::new_from_slices(&key, &iv).unwrap();
            decrypted = cipher.decrypt_vec(&decrypted).unwrap();
        }
        String::from_utf8(decrypted).unwrap()
    }

    // Insert a key-value pair with optional encryption
    pub async fn insert(&self, key: &str, value: &str) {
        let mut final_value = value.to_string();

        if self.config.encryption_enabled {
            let encrypted = self.encrypt_value(value);
            final_value = base64::encode(encrypted);
        }

        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(key.to_string(), final_value.clone());
        }

        self.db.insert(key, final_value.as_bytes()).expect("Insert failed");
        info!("Inserted key: {}", key);
    }

    // Retrieve a value, optionally decrypting it
    pub async fn get(&self, key: &str) -> Option<String> {
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(value) = cache.get(key) {
                info!("Cache hit for key: {}", key);
                return Some(value.clone());
            }
        }

        if let Some(ivec) = self.db.get(key).expect("Get failed") {
            let value = String::from_utf8(ivec.to_vec()).unwrap();

            let final_value = if self.config.encryption_enabled {
                let decoded = base64::decode(value).unwrap();
                self.decrypt_value(&decoded)
            } else {
                value
            };

            let mut cache = self.cache.lock().unwrap();
            cache.put(key.to_string(), final_value.clone());

            info!("Cache miss, fetched from DB: {}", key);
            return Some(final_value);
        }

        None
    }

    // Advanced query: Range queries
    pub async fn range_query(&self, start: &str, end: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        for result in self.db.range(start..=end) {
            let (key, value) = result.unwrap();
            let key_str = String::from_utf8(key.to_vec()).unwrap();
            let value_str = String::from_utf8(value.to_vec()).unwrap();
            results.push((key_str, value_str));
        }
        results
    }

    // Advanced query: Pattern matching on keys
    pub async fn pattern_match(&self, pattern: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        let regex = Regex::new(pattern).unwrap();

        for result in self.db.iter() {
            let (key, value) = result.unwrap();
            let key_str = String::from_utf8(key.to_vec()).unwrap();

            if regex.is_match(&key_str) {
                let value_str = String::from_utf8(value.to_vec()).unwrap();
                results.push((key_str, value_str));
            }
        }
        results
    }

    // Transaction support
    #[allow(dead_code)]
    pub async fn transaction(&self, operations: Vec<(&str, Option<&str>)>) {
        let db = Arc::clone(&self.db);

        db.transaction(|tx_db| {
            for (key, value) in operations.clone() {
                match value {
                    Some(val) => tx_db.insert(key, val.as_bytes())?,
                    None => tx_db.remove(key)?,
                };
            }
            Ok::<(), sled::transaction::ConflictableTransactionError>(())
        }).expect("Transaction failed");

        info!("Transaction executed");
    }

    // Delete a key-value pair
    pub async fn delete(&self, key: &str) {
        self.db.remove(key).expect("Delete failed");
        {
            let mut cache = self.cache.lock().unwrap();
            cache.pop(key);
        }
        info!("Deleted key: {}", key);
    }
}

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