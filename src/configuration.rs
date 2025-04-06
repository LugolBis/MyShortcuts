use std::io::Write;
use std::{env, process::Command};
use std::fs::OpenOptions;

const LINUX_TERMINAL: [&str;29] = ["x-terminal-emulator", "mate-terminal", "gnome-terminal", 
    "terminator", "xfce4-terminal", "urxvt", "rxvt", "termit", "Eterm", "aterm", "uxterm", "xterm", "roxterm",
    "termite", "lxterminal", "terminology", "st", "qterminal", "lilyterm", "tilix", "terminix", "konsole", "kitty",
    "guake", "tilda", "alacritty", "hyper", "wezterm", "rio"];

pub fn configure() {
    let mut config = String::new();

    match env::consts::OS {
        "windows" => {},
        "macos" => {},
        "linux" => {
            if let Ok(terminal) = get_linux_terminal() {
                config.push_str(&format!("{}\nlinux",terminal));
            }
        },
        unknow => {
            config.push_str(&format!("Not Supported OS : {}",unknow));
        }
    }

    if let Ok(current_dir) = env::current_dir() {
        let path = format!("{}/shell_script/config.txt",current_dir.display());
        if let Ok(mut file) = OpenOptions::new().write(true).create(true).truncate(true).open(path) {
            let _ = file.write_all(config.as_bytes());
        }
    }
}

fn get_linux_terminal() -> Result<String,()> {
    let mut terminals = LINUX_TERMINAL.to_vec();
    let env_var = env::var("TERMINAL").unwrap_or_default();
    terminals.push(&env_var);

    for terminal in terminals {
        if !terminal.is_empty() && Command::new("which").arg(&terminal).output().is_ok() {
            return Ok(String::from(terminal));
        }
    }
    Err(())
}