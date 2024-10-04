use crate::config::VibraConfig;
use crate::models::Row;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use rand::Rng;
use sled::Db;
use tokio;
use std::sync::{Arc, Mutex};
#[allow(unused_imports)]
use log::{info, error};
use lru::LruCache;
use std::str;
use std::fs;
use aes_gcm::aead::generic_array::typenum::U12;
use tokio::task;
use rayon::prelude::*;
use std::sync::RwLock;

const AES_LAYERS: usize = 25; // 25 layers of encryption

#[derive(Clone)]
pub struct VibraDB {
    db: Arc<Db>,
    cache: Arc<RwLock<LruCache<String, String>>>,
    path: String,
}

/// `VibraDB` is a database abstraction that provides functionalities for creating, managing, and interacting with a database.
/// It supports encryption with multiple layers of AES, caching, and asynchronous operations.
///
/// # Methods
///
/// - `new(config: VibraConfig) -> VibraDB`
///   - Creates a new instance of `VibraDB` with custom configurations.
///
/// - `generate_key() -> Key<Aes256Gcm>`
///   - Generates a random AES256 key.
///
/// - `generate_nonce() -> Nonce<U12>`
///   - Generates a random nonce.
///
/// - `encrypt_value(&self, value: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>)`
///   - Encrypts a value with 25 layers of AES encryption.
///
/// - `decrypt_value(&self, encrypted_data: &[u8], key: &[u8], nonce: &[u8]) -> Result<String, String>`
///   - Decrypts a value with 25 layers of AES decryption.
///
/// - `create_table(&self, table_name: &str)`
///   - Creates a new table in the database.
///
/// - `delete_table(&self, table_name: &str)`
///   - Deletes a table from the database.
///
/// - `insert_row(&self, table_name: &str, row: Row)`
///   - Inserts a row into a table.
///
/// - `insert_rows(&self, table_name: &str, rows: Vec<Row>)`
///   - Inserts multiple rows into a table.
///
/// - `get_row(&self, table_name: &str, row_id: &str) -> Option<Row>`
///   - Retrieves a row from a table.
///
/// - `update_row(&self, table_name: &str, row: Row)`
///   - Updates a row in a table.
///
/// - `delete_row(&self, table_name: &str, row_id: &str)`
///   - Deletes a row from a table.
///
/// - `truncate_table(&self, table_name: &str)`
///   - Truncates a table, removing all its rows.
///
/// - `truncate_db(&self)`
///   - Truncates the entire database, removing all data.
///
/// - `delete_db(&self)`
///   - Deletes the entire database, including its directory.
impl VibraDB {
    // Create a new instance of VibraDB with custom configurations
    pub fn new(config: VibraConfig) -> VibraDB {
        let db_path = config.path.as_ref().expect("Config path is None");
        let db = sled::open(db_path).expect("Failed to open VibraDB");
        info!("VibraDB initialized at {:?}", config.path);
        let cache = LruCache::new(config.cache_size.expect("Cache size is None"));
        let lpath = config.path.clone().expect("Config path is None") + "/";
        let rpath = ".gitignore".to_string();
        let path = lpath + &rpath;
        fs::write(path, b"*\n").expect("Failed to write .gitignore");
        VibraDB {
            db: Arc::new(db),
            cache: Arc::new(RwLock::new(cache)),
            path: config.path.expect("Config path is None"),
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

    // Encrypt value with 25 layers of AES
    fn encrypt_value(&self, value: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let encrypted_data = value.as_bytes().to_vec();
        let key = Mutex::new(vec![0u8; AES_LAYERS * 32]);
        let nonce = Mutex::new(vec![0u8; AES_LAYERS * 12]);

        let encrypted_data = (0..AES_LAYERS).into_par_iter().fold(
            || encrypted_data.clone(),
            |mut data, i| {
                let k = Self::generate_key();
                let cipher = Aes256Gcm::new(&k);
                let n = Self::generate_nonce();
                data = cipher.encrypt(&n, data.as_ref()).expect("Encryption failed");

                {
                    let mut key_guard = key.lock().unwrap();
                    key_guard[i * 32..(i + 1) * 32].copy_from_slice(k.as_slice());
                }

                {
                    let mut nonce_guard = nonce.lock().unwrap();
                    nonce_guard[i * 12..(i + 1) * 12].copy_from_slice(n.as_slice());
                }

                data
            },
        ).reduce(|| encrypted_data.clone(), |a, _| a);

        let key = key.into_inner().unwrap();
        let nonce = nonce.into_inner().unwrap();

        (encrypted_data, key, nonce)
    }

    // Decrypt value with 25 layers of AES
    fn decrypt_value(&self, encrypted_data: &[u8], key: &[u8], nonce: &[u8]) -> Result<String, String> {
        let data = encrypted_data.to_vec();

        let data = (0..AES_LAYERS).into_par_iter().fold(
            || data.clone(),
            |mut data, i| {
                let k = Key::<Aes256Gcm>::from_slice(&key[i * 32..(i + 1) * 32]);
                let cipher = Aes256Gcm::new(k);
                let n = Nonce::<U12>::from_slice(&nonce[i * 12..(i + 1) * 12]);
                data = match cipher.decrypt(n, data.as_ref()) {
                    Ok(decrypted_data) => decrypted_data,
                    Err(_) => return data, // Return the current data in case of decryption failure
                };
                data
            },
        ).reduce(|| data.clone(), |a, _| a);

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
            let mut cache = self.cache.write().unwrap();
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

    // Insert rows into a table
    #[allow(dead_code)]
    pub async fn insert_rows(&self, table_name: &str, rows: Vec<Row>) {
        for row in rows {
            self.insert_row(table_name, row).await;
        }
    }

    // Retrieve a row from a table
    pub async fn get_row(&self, table_name: &str, row_id: &str) -> Option<Row> {
        let key = format!("{}/{}", table_name, row_id);
        {
            let mut cache = self.cache.write().unwrap();
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
                    let mut cache = self.cache.write().unwrap();
                    cache.put(String::from_utf8(key.to_vec()).expect("Invalid UTF-8 sequence"), decrypted_value.clone());
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
                let mut cache = cache.write().unwrap();
                cache.pop(key.as_str());
            }
            info!("Deleted row from table {}: {}", table_name_clone, row_id_clone);
        }).await.unwrap();
    }

    // Truncate a table
    #[allow(dead_code)]
    pub async fn truncate_table(&self, table_name: &str) {
        let table_name = table_name.to_string();
        let db = self.db.clone();
        let cache = self.cache.clone();
        task::spawn_blocking(move || {
            let mut cache = cache.write().unwrap();
            let mut keys_to_remove = vec![];
            for key in cache.iter() {
                if key.0.starts_with(&table_name) {
                    keys_to_remove.push(key.0.clone());
                }
            }
            for key in keys_to_remove {
                cache.pop(&key);
            }
            let mut keys_to_remove = vec![];
            for key in db.iter() {
                if let Ok((k, _)) = key {
                    if let Ok(key_str) = str::from_utf8(&k) {
                        if key_str.starts_with(&table_name) {
                            keys_to_remove.push(key_str.to_string());
                        }
                    }
                }
            }
            for key in keys_to_remove {
                db.remove(key.as_bytes()).expect("Truncate table failed");
            }
            info!("Truncated table: {}", table_name);
        }).await.unwrap();
    }

    // Truncate DB
    #[allow(dead_code)]
    pub async fn truncate_db(&self) {
        let db = self.db.clone();
        let cache = self.cache.clone();
        task::spawn_blocking(move || {
            let mut cache = cache.write().unwrap();
            cache.clear();
            db.clear().expect("Truncate DB failed");
            info!("Truncated DB");
        }).await.unwrap();
    }

    // Delete DB
    #[allow(dead_code)]
    pub async fn delete_db(&self) {
        // Close the DB first
        drop(self.db.clone());

        // Delete the DB directory
        let path = &self.path;
        fs::remove_dir_all(path).expect("Delete DB failed");
    }
}