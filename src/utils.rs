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
    #[macro_export]
    macro_rules! result_string {
        ($text: expr) => {
            String::from($text.replace(";"," ").trim_end())
        };
    }

    /// This macro take 3 arguments : a text, a pattern (used to split the text),<br>
    /// a boolean (used to specified if you need the specific name format)
    #[macro_export]
    macro_rules! result_vec {
        ($text: expr, $pattern: expr, $is_name: expr) => {
            if $is_name { $text.split($pattern).filter(|e| !e.is_empty() && e!=&"\n").map(|e| result_string!(e)).collect::<Vec<String>>() }
            else { $text.split($pattern).filter(|e| !e.is_empty() && e!=&"\n").map(String::from).collect::<Vec<String>>() }
        };
    }

    /// This macro generate a bash command to connect to neo4j<br>
    /// from a vector of String or &str
    #[macro_export]
    macro_rules! neo4j {
        ($vector: expr) => {
            if $vector.len() == 4 { format!("cypher-shell -a neo4j://{}:{} -u {} -p '{}'",$vector[0],$vector[1],$vector[2],$vector[3]) }
            else if $vector.len() == 5 { format!("cypher-shell -a neo4j://{}:{} -u {} -p '{}' -d {}",$vector[0],$vector[1],$vector[2],$vector[3],$vector[4]) }
            else {String::from("")}
        };
    }
}

#[test]
fn test_macro() {
    use super::*;
    assert_eq!(result_string!("tutu;lulu;"), String::from("tutu lulu"));

    assert_eq!(result_vec!("tutu;lulu;",";",false), vec![String::from("tutu"),String::from("lulu")]);
    assert_eq!(result_vec!("tutu;lulu;\njuju;jojo;","\n",true), vec![String::from("tutu lulu"),String::from("juju jojo")]);

    assert_eq!(neo4j!(vec!["localhost","7687","userA","password"]),String::from("cypher-shell -a neo4j://localhost:7687 -u userA -p 'password'"));
    assert_eq!(neo4j!(vec!["localhost","7687","userA","password","my_db"]),String::from("cypher-shell -a neo4j://localhost:7687 -u userA -p 'password' -d my_db"));
}