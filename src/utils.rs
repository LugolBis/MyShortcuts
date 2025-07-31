use std::path::PathBuf;
use std::process::Command;
use std::fs::OpenOptions;
use std::io::{Write, Read};
use std::env;
use std::thread;

use crate::shell_scripts::{COMMAND, MAIN};

pub struct Logs;

impl Logs {
    pub fn write(content: String) {
        let mut path = get_folder_path().unwrap();
        path.push("log.txt"); 
        if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(path) {
            let _ = file.write(content.as_bytes());
        }
    }
}

pub fn get_folder_path() -> Result<PathBuf, String> {
    let mut exe_path = env::current_exe()
        .map_err(|e| format!("{}", e))?;
    exe_path.pop();
    exe_path.push("myshortcuts_resources");
    Ok(exe_path)
}

pub fn run_command() {
    match env::consts::OS {
        "linux" | "macos" => run_bash(),
        "windows" => run_powershell(),
        unsupported => Logs::write(format!("\nERROR : Unsupported OS '{}'",unsupported)),
    }
}

fn run_powershell() {
    thread::spawn(move || {
        if let Ok(_) = format_to_powershell() {
            if let Ok(folder_path) = get_folder_path() {
                let script = MAIN.replace("$MYSHORTCUTS_DIR", &format!("{}", folder_path.display()));

                let exit_status = Command::new("powershell")
                    .args(vec!["-ExecutionPolicy","Bypass","-File", &script])
                    .status();
        
                match exit_status {
                    Ok(_) => {},
                    Err(error) => Logs::write(format!("{}",error)),
                }
            }
            else {Logs::write(String::from("\nERROR : utils.rs - run_powershell():\nNo current_dir"))}
        }
    });
}

fn format_to_powershell() -> Result<(),()> {
    match OpenOptions::new().write(true).truncate(true).open("current_command.txt") {
        Ok(mut file) => {
            let mut content = String::new();
            match file.read_to_string(&mut content) {
                Ok(_) => {
                    content = content.replace(" && ", " ; ");
                    match file.write_all(content.as_bytes()) {
                        Ok(_) => Ok(()),
                        Err(error) => {
                            Logs::write(format!("ERROR : utils.rs - format_to_powershell - 3\n{}",error));
                            Err(())
                        }
                    }
                },
                Err(error) => {
                    Logs::write(format!("ERROR : utils.rs - format_to_powershell - 2nd\n{}",error));
                    Err(())
                }
            }
        },
        Err(error) => {
            Logs::write(format!("ERROR : utils.rs - format_to_powershell - 1st\n{}",error));
            Err(())
        }
    }
}

fn run_bash() {
    thread::spawn(move || {
        match get_folder_path() {
            Ok(folder_path) => {
                let script = COMMAND.replace("$MYSHORTCUTS_DIR", &format!("{}", folder_path.display()));

                let exit_status = Command::new("bash")
                    .arg("-c")
                    .arg(script)
                    .status();
                match exit_status {
                    Ok(_) => {},
                    Err(error) => Logs::write(format!("\n{}",error)),
                }
            },
            Err(error) => {
                Logs::write(error)
            }
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
}
