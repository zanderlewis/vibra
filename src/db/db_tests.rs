use super::*;
use tempfile::tempdir;
use tokio;

#[tokio::test]
async fn test_create_table() {
    let config = VibraConfig {
        path: Some(tempdir().unwrap().path().to_str().unwrap().to_string()),
        cache_size: Some(1024),
        encryption_layers: Some(10),
    };
    let db = VibraDB::new(config);

    db.create_table("test_table").await;
    assert!(db.table_exists("test_table").await);
}

#[tokio::test]
async fn test_insert_and_get_row() {
    let config = VibraConfig {
        path: Some(tempdir().unwrap().path().to_str().unwrap().to_string()),
        cache_size: Some(1024),
        encryption_layers: Some(10),
    };
    let db = VibraDB::new(config);

    db.create_table("test_table").await;

    let row = Row {
        id: "row1".to_string(),
        columns: vec![
            ("name".to_string(), "John Doe".to_string()),
            ("email".to_string(), "john.doe@example.com".to_string()),
        ],
    };

    db.insert_row("test_table", row.clone()).await;
    let retrieved_row = db.get_row("test_table", "row1").await;

    assert_eq!(retrieved_row, Some(row));
}

#[tokio::test]
async fn test_delete_table() {
    let config = VibraConfig {
        path: Some(tempdir().unwrap().path().to_str().unwrap().to_string()),
        cache_size: Some(1024),
        encryption_layers: Some(10),
    };
    let db = VibraDB::new(config);

    db.create_table("test_table").await;
    db.delete_table("test_table").await;

    assert!(!db.table_exists("test_table").await);
}

#[tokio::test]
async fn test_delete_db() {
    let config = VibraConfig {
        path: Some(tempdir().unwrap().path().to_str().unwrap().to_string()),
        cache_size: Some(1024),
        encryption_layers: Some(10),
    };
    let db = VibraDB::new(config);

    db.create_table("test_table").await;
    db.delete_db().await;

    assert!(!std::path::Path::new(&db.path).exists());
}

#[tokio::test]
async fn test_truncate_table() {
    let config = VibraConfig {
        path: Some(tempdir().unwrap().path().to_str().unwrap().to_string()),
        cache_size: Some(1024),
        encryption_layers: Some(10),
    };
    let db = VibraDB::new(config);

    db.create_table("test_table").await;

    let row = Row {
        id: "row1".to_string(),
        columns: vec![
            ("name".to_string(), "John Doe".to_string()),
            ("email".to_string(), "john.doe@example.com".to_string()),
        ],
    };

    db.insert_row("test_table", row.clone()).await;

    db.truncate_table("test_table").await;

    let retrieved_row = db.get_row("test_table", "row1").await;
    assert_eq!(retrieved_row, None);
}

#[tokio::test]
async fn test_truncate_db() {
    let config = VibraConfig {
        path: Some(tempdir().unwrap().path().to_str().unwrap().to_string()),
        cache_size: Some(1024),
        encryption_layers: Some(10),
    };
    let db = VibraDB::new(config);

    db.create_table("test_table").await;

    let row = Row {
        id: "row1".to_string(),
        columns: vec![
            ("name".to_string(), "John Doe".to_string()),
            ("email".to_string(), "john.doe@example.com".to_string()),
        ],
    };

    db.insert_row("test_table", row.clone()).await;

    db.truncate_db().await;

    let retrieved_row = db.get_row("test_table", "row1").await;
    assert_eq!(retrieved_row, None);
}

#[tokio::test]
async fn test_insert_many_rows_manual() {
    let config = VibraConfig {
        path: Some(tempdir().unwrap().path().to_str().unwrap().to_string()),
        cache_size: Some(1024),
        encryption_layers: Some(10),
    };
    let db = VibraDB::new(config);

    db.create_table("test_table").await;

    let row1 = Row {
        id: "row1".to_string(),
        columns: vec![
            ("name".to_string(), "John Doe".to_string()),
            ("email".to_string(), "john.doe@example.com".to_string()),
        ],
    };

    let row2 = Row {
        id: "row2".to_string(),
        columns: vec![
            ("name".to_string(), "Jane Doe".to_string()),
            ("email".to_string(), "jane.doe@example.com".to_string()),
        ],
    };

    db.insert_rows("test_table", vec![row1.clone(), row2.clone()]).await;

    let retrieved_row1 = db.get_row("test_table", "row1").await;
    let retrieved_row2 = db.get_row("test_table", "row2").await;

    assert_eq!(retrieved_row1, Some(row1));
    assert_eq!(retrieved_row2, Some(row2));
}

#[tokio::test]
async fn test_insert_many_rows() {
    let config = VibraConfig {
        path: Some(tempdir().unwrap().path().to_str().unwrap().to_string()),
        cache_size: Some(1024),
        encryption_layers: Some(10),
    };
    let db = VibraDB::new(config);

    db.create_table("test_table").await;

    let row1 = Row {
        id: "row1".to_string(),
        columns: vec![
            ("name".to_string(), "John Doe".to_string()),
            ("email".to_string(), "john.doe@example.com".to_string()),
        ],
    };

    let row2 = Row {
        id: "row2".to_string(),
        columns: vec![
            ("name".to_string(), "Jane Doe".to_string()),
            ("email".to_string(), "jane.doe@example.com".to_string()),
        ],
    };

    db.insert_many_rows("test_table", vec![row1.clone(), row2.clone()]).await;

    let retrieved_row1 = db.get_row("test_table", "row1").await;
    let retrieved_row2 = db.get_row("test_table", "row2").await;

    assert_eq!(retrieved_row1, Some(row1));
    assert_eq!(retrieved_row2, Some(row2));
}
