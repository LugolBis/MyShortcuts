use sqlite::Value;
use std::iter;

use crate::utils::{generate_name, get_folder_path, Logs};
pub const DB_NAME: &str = "my_shortcuts.db";

/// Used For the following databases : Oracle, PostgreSQL, Neo4j,
pub const CLASSIC_SHEME: [&str; 6] = [
    "Host",
    "Port",
    "Username",
    "Password",
    "Database",
    "Script Path",
];
/// Used for the following databases : MySQL, MariaDB
pub const SOCKET_SCHEME: [&str; 7] = [
    "Host",
    "Port",
    "Username",
    "Password",
    "Database",
    "Socket Path",
    "Script Path",
];
/// Used for the following dtabases : SQLite
pub const FILE_SCHEME: [&str; 2] = ["Database Path", "Script Path"];
/// Used for MongoDB only
pub const MONGODB_SCHEME: [&str; 6] = [
    "Host",
    "Port",
    "Username",
    "Password",
    "Auth Database",
    "Script Path",
];
/// Used for Redis only
pub const REDIS_SCHEME: [&str; 6] = [
    "Host",
    "Port",
    "Username",
    "Password",
    "Database Number",
    "Script Path",
];
/// Used for the shell command
pub const CUSTOM_SHEME: [&str; 1] = ["Shell Command"];
/// Used to choose what kind of shortcut you need.
pub const AVAILABLE_SHEME: [&str; 9] = [
    "Oracle",
    "MySQL",
    "MariaDB",
    "PostgreSQL",
    "SQLite",
    "Redis",
    "MongoDB",
    "Neo4j",
    "Custom",
];

pub struct Database;

impl Database {
    pub fn init() -> Result<(), String> {
        let query = "
        DROP TABLE IF EXISTS shortcuts;
        CREATE TABLE shortcuts (name TEXT primary key, configuration TEXT, type TEXT);";
        Database::query_write(query)
    }

    pub fn query_write(query: &str) -> Result<(), String> {
        let mut path = get_folder_path().map_err(|e| format!("{:?}", e))?;
        path.push(DB_NAME);

        let shortcut = sqlite::open(path).map_err(|e| format!("{e}"))?;
        shortcut.execute(query).map_err(|e| format!("{e}"))
    }

    pub fn query_read(query: &str) -> Result<String, String> {
        let mut result = String::new();
        let mut path = get_folder_path().map_err(|e| format!("{:?}", e))?;
        path.push(DB_NAME);

        let shortcut = sqlite::open(path).map_err(|e| format!("{e}"))?;

        let mut cursor = shortcut
            .prepare(query)
            .map_err(|e| format!("{e}"))?
            .into_iter();

        while let Some(tuple) = cursor.try_next().map_err(|e| format!("{e}"))? {
            let mut line = String::new();
            for value in tuple {
                line.push_str(&format!(
                    "{};",
                    extract_value(value).map_err(|e| e.to_string())?
                ));
            }
            result.push_str(&format!("{}\n", line));
        }
        Ok(result)
    }
}

fn extract_value(value: Value) -> Result<String, String> {
    match value {
        Value::Binary(vec) => String::from_utf8(vec).map_err(|e| format!("{e}")),
        Value::Float(nb) => Ok(format!("{}", nb)),
        Value::Integer(nb) => Ok(format!("{}", nb)),
        Value::String(text) => Ok(text),
        Value::Null => Ok(String::from("null")),
    }
}

pub fn insert_default_config(current_names: Vec<String>, kind: &str, fields: usize) {
    let new_name = generate_name(current_names);
    let config: String = iter::repeat("Required")
        .take(fields)
        .collect::<Vec<_>>()
        .join(";");
    let query = format!(
        "INSERT INTO shortcuts VALUES ('{}','{}','{}');",
        new_name, config, kind
    );
    
    if let Err(error) = Database::query_write(&query) {
        Logs::write(format!(
            "\nERROR : app.rs - add_new_shortcut() :\n{}\n|-> Name generated : '{}'",
            error, new_name
        ));
    }
}
