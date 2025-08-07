#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::result::Result::Ok;
use dirs::home_dir;


fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let built_in = ["cd","pwd","echo", "exit", "type"];

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
                if (built_in.contains(&cmd)){
                    println!("{cmd} is a shell builtin");
                    continue;
                }
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
                let mut s=String::new();
                for word in &v[1..] {
                    s.push_str(word);
                    
                }
                
                let s=s.replace(['"','\''], "");
                println!("{s}");

            }
            "pwd"=>{
                match env::current_dir() {
                    Ok(dir)=>println!("{}",dir.display()),
                    Err(e)=>eprintln!("pwd: failed to get current directory{}",e),
                }
                
            }


            "exit" => {
                if v.len() == 2 && v[1] == "0" {
                    break;
                } else {
                    println!("Usage: exit 0");
                }
            }
            "cd"=>{
                
                let path=Path::new(&v[1]);
                if(v[1]=="~"){
                    
                    let path =home_dir();
                    env::set_current_dir(path.unwrap());  
                    }
                else if (path.exists()){
                    
                    
                        env::set_current_dir(path);
                    

                    
                }
                else{
                    println!("cd: {}: No such file or directory",v[1]);
                }
                
            
            }

            _ => {
                let mut app = Command::new(v[0]);
                app.args(&v[1..]);

                match app.spawn() {
                Ok(mut child) => {
                
                let _ = child.wait();
                }
                Err(_) => {
                    
                    eprintln!("{}: command not found", v[0]);
                    }   
                }
            }
        } 
        
    }
}
