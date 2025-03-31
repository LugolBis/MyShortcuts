use std::process::Command;
use std::env;
use std::thread;

pub fn run_bash() {
    if let Ok(current_dir) = env::current_dir() {
        thread::spawn(move || {
            let mut child = Command::new("x-terminal-emulator")
                .arg("-e")
                .arg(format!("{}/main.sh",current_dir.display()))
                .arg(format!("{}/current_command.txt",current_dir.display()))
                .spawn()
                .expect("ERROR when try to launch.");
            let _ = child.wait().expect("ERROR when try to wait the result.");
        });
    }
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

    /// This macro generate a bash command to connect to neo4j<br>
    /// from a vector of String or &str
    #[macro_export]
    macro_rules! neo4j {
        ($vector: expr) => {
            if $vector.len() == 4 {format!("cypher-shell -a neo4j://{}:{} -u {} -p '{}'",$vector[0],$vector[1],$vector[2],$vector[3])}
            else if $vector.len() == 5 {format!("cypher-shell -a neo4j://{}:{} -u {} -p '{}' -d {}",$vector[0],$vector[1],$vector[2],$vector[3],$vector[4])}
            else if $vector.len() == 6 {format!("cypher-shell -a neo4j://{}:{} -u {} -p '{}' -d {} -f {}",$vector[0],$vector[1],$vector[2],$vector[3],$vector[4],$vector[5])}
            else { format!("echo 'Inconsistent connection configuration {:?}'",$vector) }
        };
    }

    /// This macro generate a bash command to connect to neo4j<br>
    /// from a vector of String or &str
    #[macro_export]
    macro_rules! postgresql {
        ($vector: expr) => {
            if $vector.len() == 4 {format!("export PGPASSWORD='{}' && psql -U {} -h {} -p {}",$vector[3],$vector[2],$vector[0],$vector[1])}
            else if $vector.len() == 5 {format!("export PGPASSWORD='{}' && psql -U {} -h {} -p {} -d {}",$vector[3],$vector[2],$vector[0],$vector[1],$vector[4])}
            else if $vector.len() == 6 {format!("export PGPASSWORD='{}' && psql -U {} -h {} -p {} -d {} -f {}",$vector[3],$vector[2],$vector[0],$vector[1],$vector[4],$vector[5])}
            else { format!("echo 'Inconsistent connection configuration {:?}'",$vector) }
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
    assert_eq!(neo4j!(vec!["localhost","7687","userA","password"]),String::from("cypher-shell -a neo4j://localhost:7687 -u userA -p 'password'"));
    assert_eq!(neo4j!(vec!["localhost","7687","userA","password","my_db"]),String::from("cypher-shell -a neo4j://localhost:7687 -u userA -p 'password' -d my_db"));
}