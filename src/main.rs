mod app;
mod database;
mod objects;
mod shell_scripts;
mod ui;
mod utils;

use app::main_app;
use database::{DB_NAME, Database};
use std::process::Command;
use std::{env, fs};
use utils::{Logs, get_folder_path};

use crate::shell_scripts::TERMINAL;

fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

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
                    insert into shortcuts values ('c6', '127.0.0.1, userA, my_db, password', 'Neo4j');
                ");
            }
        }
    } else {
        println!("ERROR : Failed to get the folder path where the script is.")
    }

    match (env::consts::OS, env::var("MYSHORTCUTSLAUNCH")) {
        ("linux" | "macos", Err(_)) => {
            let exit_status = Command::new("bash").arg("-c").arg(TERMINAL).status();
            match exit_status {
                Ok(_) => {}
                Err(error) => Logs::write(format!("\n{}", error)),
            }
        }
        (_, _) => match main_app() {
            Ok(_) => {}
            Err(error) => println!("ERROR with the function mainApp :\n{error}"),
        },
    }
}
