mod database;
mod utils;
mod app;

use database::Database;
use utils::*;

fn main() {
    use app::main;
    match main() {
        Ok(_) => {},
        Err(error) => println!("ERROR : {error}")
    }
}

#[test]
fn test_bash() {
    match run_bash() {
        Ok(res) => println!("Successfuly exit : {res}"),
        Err(res) => println!("ERROR when exit : {res}")
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
        Ok(res) => println!("Successfully run the queries."),
        Err(res) => println!("ERROR when try to run the queries :\n{res}")
    }

    match Database::query_read("SELECT * FROM test") {
        Ok(res) => println!("{res}"),
        Err(res) => println!("{res}")
    }
}