#[derive(Clone)]
#[allow(dead_code)]
pub struct Column {
    pub name: String,
    pub data_type: String,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub id: String,
    pub columns: Vec<(String, String)>, // (column_name, value)
}