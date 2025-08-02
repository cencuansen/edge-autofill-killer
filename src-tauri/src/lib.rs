use rusqlite::types::Value as SqlValue;
use rusqlite::Connection;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
enum DbError {
    #[error("Database error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Table not found: {0}")]
    TableNotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Database not connected")]
    NotConnected,
}

impl serde::Serialize for DbError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

struct DbConnection {
    conn: Mutex<Option<Connection>>,
}

impl DbConnection {
    fn new() -> Self {
        Self {
            conn: Mutex::new(None),
        }
    }

    fn open(&self, path: &str) -> Result<(), DbError> {
        if !Path::new(path).exists() {
            return Err(DbError::NotFound(path.to_string()));
        }

        let mut conn = self.conn.lock().unwrap();
        *conn = Some(Connection::open(path)?);
        Ok(())
    }

    // fn close(&self) {
    //     let mut conn = self.conn.lock().unwrap();
    //     *conn = None;
    // }

    fn get_conn(&self) -> Result<std::sync::MutexGuard<Option<Connection>>, DbError> {
        let conn = self.conn.lock().unwrap();
        if conn.is_none() {
            return Err(DbError::NotConnected);
        }
        Ok(conn)
    }
}

lazy_static::lazy_static! {
    static ref DB_CONN: DbConnection = DbConnection::new();
}

fn app_data_local() -> String {
    format!(
        "{}/Microsoft/Edge/User Data/Default/Web Data",
        env::var("LOCALAPPDATA").unwrap()
    )
}

lazy_static::lazy_static! {
    static ref TARGET_PATH: String = app_data_local();
}

#[tauri::command]
async fn get_table_data(table_name: String) -> Result<Vec<HashMap<String, Value>>, DbError> {
    DB_CONN.open(&TARGET_PATH)?;

    let conn_guard = DB_CONN.get_conn()?;
    let conn = conn_guard.as_ref().ok_or(DbError::NotConnected)?;

    // 检查表是否存在
    let table_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?",
        [&table_name],
        |row| row.get(0),
    )?;

    if table_exists == 0 {
        return Err(DbError::TableNotFound(table_name));
    }

    // 获取所有列名
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
    let columns: Vec<String> = stmt
        .query_map([], |row| Ok(row.get::<_, String>(1)?))?
        .collect::<Result<Vec<_>, _>>()?;

    // 查询数据
    let query = format!("SELECT * FROM {}", table_name);
    let mut stmt = conn.prepare(&query)?;

    let rows = stmt
        .query_map([], |row| {
            let mut map = HashMap::new();
            for (i, col) in columns.iter().enumerate() {
                let value: SqlValue = row.get(i)?;
                let json_value = match value {
                    SqlValue::Null => Value::Null,
                    SqlValue::Integer(i) => Value::from(i),
                    SqlValue::Real(f) => Value::from(f),
                    SqlValue::Text(s) => Value::from(s),
                    SqlValue::Blob(b) => Value::from(b), // 二进制数据转为Base64字符串
                };
                map.insert(col.clone(), json_value);
            }
            Ok(map)
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

#[tauri::command]
async fn delete_record(table_name: String, name: String, value: String) -> Result<bool, DbError> {
    let conn_guard = DB_CONN.get_conn()?;
    let conn = conn_guard.as_ref().ok_or(DbError::NotConnected)?;

    let changes = conn.execute(
        &format!("DELETE FROM {} WHERE name = ? AND value = ?", table_name),
        [name, value],
    )?;

    Ok(changes > 0)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_table_data, delete_record])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
