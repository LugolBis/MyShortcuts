use std::process::Command;
use std::fs::OpenOptions;
use std::io::Write;
use std::env;
use std::thread;

pub struct Logs;

impl Logs {
    pub fn write(content: String) {
        if let Ok(mut file) = OpenOptions::new().write(true).create(true).open("log.txt") {
            let _ = file.write(content.as_bytes());
        }
    }
}

pub fn run_command() {
    match env::consts::OS {
        "linux" | "macos" => run_bash(),
        "windows" => run_powershell(),
        unsupported => Logs::write(format!("ERROR : Unsupported OS '{}'",unsupported)),
    }
}

fn run_powershell() {
    todo!("NOT TESTED FEATURE");
}

fn run_bash() {
    let script = r#"
    {
        counter_window=$(tmux list-windows -t myshortcuts | wc -l)
        current_command=$(cat current_command.txt)
        tmux new-window -t myshortcuts
        tmux send-keys -t myshortcuts:$counter_window "$current_command" C-m
    } > log.txt 2>&1
    "#;

    thread::spawn(move || {
        let exit_status = Command::new("bash")
            .arg("-c")
            .arg(script)
            .status();
        match exit_status {
            Ok(_) => {},
            Err(error) => Logs::write(format!("{}",error)),
        }
    });
}

pub fn neo4j(vector: Vec<&String>) -> String {
    let mut flags = Vec::new();
    if let (Some(&host), Some(&port)) = (vector.get(0), vector.get(1)) {
        if !host.is_empty() && !port.is_empty() {
            flags.push(format!("-a neo4j://{}:{}", host, port));
        }
    }
    for &(index, flag) in &[(2, "-u"), (3, "-p"), (4, "-d"), (5, "-f")] {
        if let Some(&value) = vector.get(index) {
            if !value.is_empty() {
                flags.push(format!("{} {}", flag, value));
            }
        }
    }
    format!("cypher-shell {}", flags.join(" "))
}

pub fn postgresql(vector: Vec<&String>) -> String {
    let mut command = Vec::new();

    if let Some(password) = vector.get(3).filter(|s| !s.is_empty()) {
        command.push(format!("export PGPASSWORD='{}' &&", password));
    }
    command.push("psql".to_string());

    for &(index, flag) in &[(0, "-h"), (1, "-p"), (2, "-U"), (4, "-d"), (5, "-f")] {
        if let Some(value) = vector.get(index).filter(|s| !s.is_empty()) {
            command.push(format!("{} {}", flag, value));
        }
    }
    command.join(" ")
}

pub fn mysql(vector: Vec<&String>) -> String {
    let mut flags = Vec::new();

    for &(index, flag) in &[(0, "-h"), (1, "-P"), (2, "-u")] {
        if let Some(value) = vector.get(index).filter(|s| !s.is_empty()) {
            flags.push(format!("{} {}", flag, value));
        }
    }
    if let Some(value) = vector.get(3).filter(|s| !s.is_empty()) {
        flags.push(format!("-p'{}'",value));
    }
    if let Some(value) = vector.get(5).filter(|s| !s.is_empty()) {
        flags.push(format!("--protocol=socket -S {}",value))
    }
    if let Some(value) = vector.get(4).filter(|s| !s.is_empty()) {
        flags.push(String::clone(value));
    }
    if let Some(value) = vector.get(6).filter(|s| !s.is_empty()) {
        flags.push(format!("< {}",value));
    }
    format!("mysql {}",flags.join(" "))
}

pub fn mariadb(vector: Vec<&String>) -> String {
    mysql(vector).replace("mysql", "mariadb")
}

pub fn sqlite(vector: Vec<&String>) -> String {
    match (vector.get(0).filter(|s| !s.is_empty()), vector.get(1).filter(|s| !s.is_empty())) {
        (Some(db_path), Some(script_path)) => format!("sqlite3 {} < {}",db_path,script_path),
        (Some(db_path), None) => format!("sqlite3 {}",db_path),
        _ => format!("echo 'Inconsistent SQLite arguments : {:?}'",vector)
    }
}

pub fn oracle(vector: Vec<&String>) -> String {
    let mut command = String::new();
    match (vector.get(0).filter(|s| !s.is_empty()), vector.get(1).filter(|s| !s.is_empty())) {
        (Some(host),Some(port)) => {
            match (vector.get(2).filter(|s| !s.is_empty()), vector.get(3).filter(|s| !s.is_empty())) {
                (Some(username),Some(password)) => {
                    command.push_str(&format!("sqlplus {}/{}@{}:{}",username,password,host,port));
                },
                _ => { return format!("echo 'Inconsistent Oracle arguments : {:?}'",vector) }
            }
        },
        _ => { return format!("echo 'Inconsistent Oracle arguments : {:?}'",vector) }
    }

    if let Some(database) = vector.get(4).filter(|s| !s.is_empty()) {
        command.push_str(&format!("/{}",database));
    }
    if let Some(script_path) = vector.get(5).filter(|s| !s.is_empty()) {
        command = command.replace("sqlplus","echo exit | sqlplus -s ");
        command.push_str(&format!(" @{}",script_path));
    }
    command
}

pub fn mongodb(vector: Vec<&String>) -> String {
    let mut flags = Vec::new();
    for &(index, flag) in &[(0, "--host"),(1, "--port"),(2, "-u"),(3, "-p"),(4,"--authenticationDatabase"),(5,"-f")] {
        if let Some(value) = vector.get(index).filter(|s| !s.is_empty()) {
            flags.push(format!("{} {}", flag, value));
        }
    }
    format!("mongosh {}", flags.join(" "))
}

pub fn redis(vector: Vec<&String>) -> String {
    let mut flags = Vec::new();
    for &(index, flag) in &[(0, "-h"),(1, "-p"),(2, "--user"),(3, "-a"),(4,"-n"),(5,"--eval")] {
        if let Some(value) = vector.get(index).filter(|s| !s.is_empty()) {
            flags.push(format!("{} {}", flag, value));
        }
    }
    format!("redis-cli {}", flags.join(" "))
}

#[macro_use]
pub mod macros {
    /// This macro take in argument a vector of String and format them into<br>
    /// a String that contains the configuration in a specific format to be store<br>
    /// in the database
    #[macro_export]
    macro_rules! format_config {
        ($vector: expr) => {
            $vector.into_iter().map(|e| format!("{};",e.get_value())).collect::<String>()
        };
    }

    #[macro_export]
    macro_rules! filter_config {
        ($vector: expr) => {
            {
                let mut result = $vector;
                let mut last_index = result.len()-1;
                while result[last_index] == "" || result[last_index] == "\n"  {
                    result.pop();
                    last_index -= 1usize;
                }
                result
            }
        };
    }

    /// This macro take 3 arguments : a text, a pattern (used to split the text),<br>
    /// a boolean (used to specified if you need the specific name format)
    #[macro_export]
    macro_rules! result_vec {
        ($text: expr, $pattern: expr) => {
            $text.split($pattern).filter(|e| !e.is_empty() && e!=&"\n").map(String::from).collect::<Vec<String>>()
        };
    }
}

#[test]
fn test_macro() {
    use super::*;
    use crate::objects::Configuration;
    assert_eq!(format_config!(vec![Configuration::default()]),String::from("echo Welcome on MyShortcuts;"));
    assert_eq!(filter_config!(vec![String::from("lulu"),String::new(),String::from("tutu"),String::new(),String::new()]),vec![String::from("lulu"),String::new(),String::from("tutu")]);
    assert_eq!(result_vec!("tutu;lulu;",";"), vec![String::from("tutu"),String::from("lulu")]);
}