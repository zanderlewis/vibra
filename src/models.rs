#[derive(Clone)]
#[allow(dead_code)]
pub struct Column {
    pub name: String,
    pub data_type: String,
}

#[derive(Clone, Debug)]
/// Represents a row in a table with an identifier and a collection of columns.
///
/// # Fields
///
/// * `id` - A unique identifier for the row.
/// * `columns` - A vector of tuples where each tuple contains a column name and its corresponding value.
pub struct Row {
    pub id: String,
    pub columns: Vec<(String, String)>, // (column_name, value)
}