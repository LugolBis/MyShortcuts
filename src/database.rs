use sqlite::Value;

const DB_NAME: &str = "my_shortcuts.db";

pub struct Database;

impl Database {
    pub fn init() -> Result<(), String> {
        let query = "
        DROP TABLE IF EXISTS connections;
        CREATE TABLE connections (name TEXT primary key, configuration TEXT, type TEXT);";
        Database::query_write(query)
    }

    pub fn query_write(query:&str) -> Result<(), String> {
        let connection = sqlite::open(DB_NAME)
            .map_err(|e| format!("{e}"))?;
        connection.execute(query).map_err(|e| format!("{e}"))
    }

    pub fn query_read(query:&str) -> Result<String, String> {
        let mut result = String::new();
        let connection = sqlite::open(DB_NAME)
            .map_err(|e| format!("{e}"))?;

        let mut cursor = connection.prepare(query).map_err(|e| format!("{e}"))?.into_iter();

        while let Some(tuple) = cursor.try_next().map_err(|e| format!("{e}"))? {
            let mut line = String::new();
            for value in tuple {
                line.push_str(&format!("{:?};",extract_value(value).map_err(|e| format!("{e}"))?));
            }
            result.push_str(&format!("{}\n",line));
        }
        Ok(result)
    }
}

fn extract_value(value:Value) -> Result<String,String> {
    match value {
        Value::Binary(vec) => String::from_utf8(vec).map_err(|e| format!("{e}")),
        Value::Float(nb) => Ok(format!("{}",nb)),
        Value::Integer(nb) => Ok(format!("{}",nb)),
        Value::String(text) => Ok(text),
        Value::Null => Ok(String::from("null"))
    }
}