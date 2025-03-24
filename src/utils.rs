use std::process::Command;
use std::env;

pub fn run_bash() -> Result<String,String> {
    /// Open a new terminal and run the bash command store in "current_command.txt"
    let current_dir = env::current_dir()
        .map_err(|e| format!("{}",e))?;
    let mut child = Command::new("x-terminal-emulator")
        .arg("-e")
        .arg(format!("{}/main.sh",current_dir.display()))
        .arg(format!("{}/current_command.txt",current_dir.display()))
        .spawn().map_err(|e| format!("{}",e))?;

    match child.wait() {
        Ok(res) => Ok(format!("{}",res)),
        Err(res) => Err(format!("{}",res))
    }
}