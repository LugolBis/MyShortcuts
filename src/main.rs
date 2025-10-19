mod app;
mod database;
mod objects;
mod ui;
mod utils;

use app::main_app;
use database::{DB_NAME, Database};
use std::{fs::{self, OpenOptions}, io::Write};
use utils::{Logs, get_folder_path};

fn main() {
    if let Ok(mut path) = get_folder_path() {
        if !fs::exists(&path).unwrap_or(true) {
            match fs::create_dir(&path) {
                Ok(_) => {}
                Err(error) => Logs::write(format!("\n{:?}", error)),
            }
        }

        path.push(DB_NAME);

        if !fs::exists(path).unwrap_or(true) {
            if let Err(error) = Database::init() {
                panic!("{error}")
            } else {
                let _init = Database::query_write("
                    insert into shortcuts values ('c6', '127.0.0.1', 'userA', 'my_db', 'password', 'Neo4j');
                ");
            }
        }
    } else {
        println!("ERROR : Failed to get the folder path where the script is.")
    }

    match main_app() {
        Ok(command) => {
            if !command.is_empty() {
                match OpenOptions::new().create(true).truncate(true).write(true).open("/tmp/myshortcuts_command.sh") {
                    Ok(mut file) => {
                        if let Err(error) = file.write_all(command.as_bytes()) {
                            Logs::write(format!("\n{}", error));
                        }
                    }
                    Err(error) => {
                        Logs::write(format!("\n{}", error));
                    }
                }
            }
        },
        Err(error) => Logs::write(format!("ERROR with the function mainApp :\n{error}")),
    }
}
