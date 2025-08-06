#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        //let built_in = ["echo", "exit", "type"];

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim();
        let v: Vec<&str> = trimmed.split_whitespace().collect();

        if v.is_empty() {
            continue;
        }

         match v[0] {
            "type" => {
                if v.len() < 2 {
                    println!("type: missing operand");
                    continue;
                    
                }

                let cmd = v[1];
                let mut found = false;

                if let Ok(path_var) = env::var("PATH") {
                    for dir in path_var.split(':') {
                        let full_path = format!("{}/{}", dir, cmd);
                        let path = Path::new(&full_path);

                        if path.exists() && path.is_file() {
                            if let Ok(metadata) = fs::metadata(path) {
                                if metadata.permissions().mode() & 0o111 != 0 {
                                    println!("{cmd} is {full_path}");
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if !found {
                    println!("{cmd}: not found");
                }
            }

            "echo" => {
                for word in &v[1..] {
                    print!("{} ", word);
                }
                println!();
            }

            "exit" => {
                if v.len() == 2 && v[1] == "0" {
                    break;
                } else {
                    println!("Usage: exit 0");
                }
            }

            _ => {
                println!("{}: command not found", trimmed);
            }
        }
    }
}
