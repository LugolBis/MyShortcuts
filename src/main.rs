mod database;
mod utils;
mod objects;
mod ui;
mod app;
mod configuration;

use database::Database;
use app::main_app;
use configuration::configure;
use std::fs;

fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if !fs::exists("/shell_script/config.txt").unwrap_or(false) {
        configure()
    }

    if !fs::exists("my_shortcuts.db").unwrap_or(false) {
        if let Err(error) = Database::init() {
            panic!("{error}")
        }
        else {
            let _lulu = Database::query_write("
                insert into shortcuts values ('c1', '127.0.0.1;userA;ma_db;password', 'Neo4j');
                insert into shortcuts values ('c2', 'config2', 'MySQL');
                insert into shortcuts values ('c3', 'config3', 'Neo4j');
                insert into shortcuts values ('c4', 'config4', 'Neo4j');
            ");
        }
    }
    
    match main_app() {
        Ok(_) => {},
        Err(error) => println!("ERROR with the function mainApp :\n{error}")
    }
}

#[test]
fn test_sqlite() {
    let query = "
        drop table test;
        CREATE TABLE test (name TEXT, age INTEGER);
        INSERT INTO test VALUES ('Alice', 42);
        INSERT INTO test VALUES ('Bob', 55);";
    match Database::query_write(query) {
        Ok(_res) => println!("Successfully run the queries."),
        Err(res) => println!("ERROR when try to run the queries :\n{res}")
    }

    match Database::query_read("SELECT * FROM test") {
        Ok(res) => println!("{res}"),
        Err(res) => println!("{res}")
    }
}