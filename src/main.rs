use aes_gcm::{Aes256Gcm, Key, Nonce}; // Or `Aes128Gcm`
use aes_gcm::aead::{Aead, KeyInit};
use rand::Rng;
use sled::Db;
use tokio;
use std::sync::{Arc, Mutex};
use log::info;
use lru::LruCache;
use std::str;
use std::fs;
use aes_gcm::aead::generic_array::typenum::U12;
use tokio::task;

const AES_LAYERS: usize = 25; // 25 layers of encryption

// Configurations for the database
#[derive(Clone)]
struct VibraConfig {
    path: String,
    cache_size: usize,
}

#[derive(Clone)]
struct VibraDB {
    db: Arc<Db>,
    cache: Arc<Mutex<LruCache<String, String>>>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct Column {
    name: String,
    data_type: String,
}

#[derive(Clone, Debug)]
struct Row {
    id: String,
    columns: Vec<(String, String)>, // (column_name, value)
}

impl VibraDB {
    // Create a new instance of VibraDB with custom configurations
    pub fn new(config: VibraConfig) -> VibraDB {
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
        }
    }

    fn generate_key() -> Key<Aes256Gcm> {
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        Key::<Aes256Gcm>::from_slice(&key).clone()
    }

    fn generate_nonce() -> Nonce<U12> {
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill(&mut nonce);
        Nonce::<U12>::from_slice(&nonce).clone()
    }

    // Encrypt value with 25 layers of AES using SIMD
    fn encrypt_value(&self, value: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let mut encrypted_data = value.as_bytes().to_vec();
        let mut key = vec![];
        let mut nonce = vec![];
        for _ in 0..AES_LAYERS {
            let k = Self::generate_key();
            let cipher = Aes256Gcm::new(&k);
            let n = Self::generate_nonce();
            encrypted_data = cipher.encrypt(&n, encrypted_data.as_ref())
                .expect("Encryption failed");
            key.extend_from_slice(k.as_slice());
            nonce.extend_from_slice(n.as_slice());
        }
        (encrypted_data, key, nonce)
    }

    // Decrypt value with 25 layers of AES using SIMD
    fn decrypt_value(&self, encrypted_data: &[u8], key: &[u8], nonce: &[u8]) -> Result<String, String> {
        let mut data = encrypted_data.to_vec();
        for i in 0..AES_LAYERS {
            let k = Key::<Aes256Gcm>::from_slice(&key[i*32..(i+1)*32]);
            let cipher = Aes256Gcm::new(k);
            let n = Nonce::<U12>::from_slice(&nonce[i*12..(i+1)*12]);
            data = match cipher.decrypt(n, data.as_ref()) {
                Ok(decrypted_data) => decrypted_data,
                Err(_) => return Err("Decryption failed".to_string()),
            };
        }
        match String::from_utf8(data) {
            Ok(valid_string) => Ok(valid_string),
            Err(_) => Err("Invalid UTF-8 sequence".to_string()),
        }
    }

    // Create a new table
    pub async fn create_table(&self, table_name: &str) {
            let db = self.db.clone();
            let table_name = table_name.to_string();
            task::spawn_blocking(move || {
                db.insert(table_name.as_bytes(), b"").expect("Create table failed");
                info!("Created table: {}", table_name);
            }).await.unwrap();
        }

    // Delete a table
    #[allow(dead_code)]
    pub async fn delete_table(&self, table_name: &str) {
        let db = self.db.clone();
        let table_name = table_name.to_string();
        task::spawn_blocking(move || {
            db.remove(table_name.as_bytes()).expect("Delete table failed");
            info!("Deleted table: {}", table_name);
        }).await.unwrap();
    }

    // Insert a row into a table
    pub async fn insert_row(&self, table_name: &str, row: Row) {
        let key = format!("{}/{}", table_name, row.id);
        let data = serde_json::to_string(&row.columns).expect("Serialization failed");
        let (encrypted_value, key_data, nonce_data) = self.encrypt_value(&data);
        let mut combined_data = encrypted_value;
        combined_data.extend_from_slice(&key_data);
        combined_data.extend_from_slice(&nonce_data);
    
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(key.clone(), data.clone()); // Cache stores the plaintext
        }
    
        let db = self.db.clone();
        let key_clone = key.clone();
        let table_name_clone = table_name.to_string(); // Clone table_name here
        task::spawn_blocking(move || {
            db.insert(key_clone, combined_data).expect("Insert row failed");
            info!("Inserted row into table {}: {}", table_name_clone, row.id); // Use cloned table_name
        }).await.unwrap();
    }

    // Retrieve a row from a table
    pub async fn get_row(&self, table_name: &str, row_id: &str) -> Option<Row> {
        let key = format!("{}/{}", table_name, row_id);
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(value) = cache.get(&key) {
                info!("Cache hit for key: {}", key);
                let columns: Vec<(String, String)> = serde_json::from_str(value).expect("Deserialization failed");
                return Some(Row { id: row_id.to_string(), columns });
            }
        }

        if let Some(ivec) = self.db.get(&key).expect("Get row failed") {
            let (encrypted_data, key_nonce) = ivec.split_at(ivec.len() - (AES_LAYERS * (32 + 12)));
            let (key, nonce) = key_nonce.split_at(AES_LAYERS * 32);
            match self.decrypt_value(encrypted_data, key, nonce) {
                Ok(decrypted_value) => {
                    let columns: Vec<(String, String)> = serde_json::from_str(&decrypted_value).expect("Deserialization failed");
                    let mut cache = self.cache.lock().unwrap();
                    cache.put(String::from_utf8_lossy(key).to_string(), decrypted_value.clone());
                    info!("Cache miss, fetched from DB and decrypted: {:?}", key);
                    Some(Row { id: row_id.to_string(), columns })
                },
                Err(err) => {
                    info!("Failed to decrypt value for key {:?}: {}", key, err);
                    None
                }
            }
        } else {
            None
        }
    }

    // Update a row in a table
    #[allow(dead_code)]
    pub async fn update_row(&self, table_name: &str, row: Row) {
        self.delete_row(table_name, &row.id).await;
        self.insert_row(table_name, row).await;
    }

    // Delete a row from a table
    pub async fn delete_row(&self, table_name: &str, row_id: &str) {
        let key = format!("{}/{}", table_name, row_id);
        let table_name_clone = table_name.to_string();
        let db = self.db.clone();
        let cache = self.cache.clone();
        let row_id_clone = row_id.to_string();
        task::spawn_blocking(move || {
            db.remove(&key).expect("Delete row failed");
            {
                let mut cache = cache.lock().unwrap();
                cache.pop(&key);
            }
            info!("Deleted row from table {}: {}", table_name_clone, row_id_clone);
        }).await.unwrap();
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