
use std::env;
use std::fs;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path};
use std::process::{Command, Stdio};

use dirs::home_dir;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter};
use rustyline::history::FileHistory;
use rustyline::{Editor, Helper, Context, Config};
use rustyline::validate::{Validator, ValidationContext, ValidationResult};

// Parsing with support for single and double quotes, escaping
fn parse_single_quotes(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '\0';
    let perserved = ['"', '\\', '$', '`'];
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next_char) = chars.next() {
                    if in_quotes {
                        if perserved.contains(&next_char) {
                            current.push(next_char);
                        } else {
                            current.push('\\');
                            current.push(next_char);
                        }
                    } else {
                        current.push(next_char);
                    }
                }
            }
            '\'' | '"' => {
                if !in_quotes {
                    in_quotes = true;
                    quote_char = c;
                } else if quote_char == c {
                    in_quotes = false;
                    quote_char = '\0';
                } else {
                    current.push(c);
                }
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next_char) = chars.next() {
                match next_char {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '\'' => result.push('\''),
                    '"' => result.push('"'),
                    '0'..='7' => {
                        let mut octal = String::new();
                        octal.push(next_char);
                        for _ in 0..2 {
                            if let Some(&next) = chars.peek() {
                                if next >= '0' && next <= '7' {
                                    octal.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                        }
                        if let Ok(val) = u8::from_str_radix(&octal, 8) {
                            result.push(val as char);
                        }
                    }
                    _ => result.push(next_char),
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

// Shell helper with completion, hint, highlight, and validation traits
struct ShellCompleter {
    builtins: Vec<String>,
}

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let start = line[..pos].rfind(' ').map_or(0, |i| i + 1);
        let word = &line[start..pos].to_lowercase();

        let mut matches = Vec::new();

        for b in &self.builtins {
            if b.starts_with(word) {
                matches.push(Pair {
                    display: b.clone(),
                    replacement: b.clone(),
                });
            }
        }

        if !word.is_empty() {
            if let Ok(entries) = std::fs::read_dir(".") {
                for entry in entries.flatten() {
                    if let Ok(fname) = entry.file_name().into_string() {
                        if fname.to_lowercase().starts_with(word) {
                            matches.push(Pair {
                                display: fname.clone(),
                                replacement: fname,
                            });
                        }
                    }
                }
            }
        }

        Ok((start, matches))
    }
}

impl Hinter for ShellCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for ShellCompleter {}

impl Validator for ShellCompleter {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for ShellCompleter {}

fn main() {
    let config = Config::builder()
        .auto_add_history(true)
        .build();

    let mut rl = Editor::<ShellCompleter, FileHistory>::with_config(config)
        .expect("Failed to create rustyline Editor");

    rl.set_helper(Some(ShellCompleter {
        builtins: vec![
            "cd".into(),
            "pwd".into(),
            "echo".into(),
            "exit".into(),
            "type".into(),
        ],
    }));

    let history_path = home_dir().map(|p| p.join(".rusty_shell_history"));
    if let Some(ref path) = history_path {
        if let Err(e) = rl.load_history(path) {
            eprintln!("Warning: could not load history: {}", e);
        }
    }

    let built_in = ["cd", "pwd", "echo", "exit", "type"];

    loop {
        let readline = rl.readline("$ ");
        let line = match readline {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("exit");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Args parsed with quotes
        let args_raw = parse_single_quotes(trimmed);
        let args: Vec<String> = args_raw.into_iter().map(|arg| unescape_string(&arg)).collect();

        if args.is_empty() {
            continue;
        }

        match args[0].as_str() {
            "history" => {
                for (idx, entry) in rl.history().iter().enumerate() {
                    println!("{}  {}", idx + 1, entry);
                }
            }
            "type" => {
                if args.len() < 2 {
                    println!("type: missing operand");
                    continue;
                }

                let cmd = &args[1];
                if built_in.contains(&cmd.as_str()) {
                    println!("{} is a shell builtin", cmd);
                    continue;
                }

                let mut found = false;
                if let Ok(path_var) = env::var("PATH") {
                    for dir in path_var.split(':') {
                        let full_path = format!("{}/{}", dir, cmd);
                        let path = Path::new(&full_path);
                        if path.exists() && path.is_file() {
                            if let Ok(metadata) = fs::metadata(path) {
                                if metadata.permissions().mode() & 0o111 != 0 {
                                    println!("{} is {}", cmd, full_path);
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                if !found {
                    println!("{}: not found", cmd);
                }
            }

            "echo" => {
                println!("{}", args[1..].join(" "));
            }

            "pwd" => {
                match env::current_dir() {
                    Ok(dir) => println!("{}", dir.display()),
                    Err(e) => eprintln!("pwd: failed to get current directory: {}", e),
                }
            }

            "exit" => {
                if args.len() == 2 && args[1] == "0" {
                    break;
                } else {
                    println!("Usage: exit 0");
                }
            }

            "cd" => {
                if args.len() < 2 {
                    println!("cd: missing operand");
                    continue;
                }
                let target = &args[1];
                if target == "~" {
                    if let Some(home) = home_dir() {
                        if let Err(e) = env::set_current_dir(home) {
                            eprintln!("cd: failed to change directory: {}", e);
                        }
                    } else {
                        eprintln!("cd: HOME not set");
                    }
                } else {
                    let path = Path::new(target);
                    if path.exists() {
                        if let Err(e) = env::set_current_dir(path) {
                            eprintln!("cd: failed to change directory: {}", e);
                        }
                    } else {
                        println!("cd: {}: No such file or directory", target);
                    }
                }
            }

            cmd => {
                // Handle simple redirections >, >>, <

                let mut command_args = Vec::new();
                let mut input_redirect: Option<String> = None;
                let mut output_redirect: Option<(String, bool)> = None; // (file, append?)
                let mut iter = args[1..].iter();

                while let Some(arg) = iter.next() {
                    if arg == ">" {
                        if let Some(file) = iter.next() {
                            output_redirect = Some((file.clone(), false));
                        } else {
                            eprintln!("syntax error near unexpected token `newline`");
                            break;
                        }
                    } else if arg == ">>" {
                        if let Some(file) = iter.next() {
                            output_redirect = Some((file.clone(), true));
                        } else {
                            eprintln!("syntax error near unexpected token `newline`");
                            break;
                        }
                    } else if arg == "<" {
                        if let Some(file) = iter.next() {
                            input_redirect = Some(file.clone());
                        } else {
                            eprintln!("syntax error near unexpected token `newline`");
                            break;
                        }
                    } else {
                        command_args.push(arg.clone());
                    }
                }

                let mut command = Command::new(cmd);
                command.args(&command_args);

                if let Some(infile) = input_redirect {
                    match fs::File::open(infile) {
                        Ok(f) => {
                            command.stdin(Stdio::from(f));
                        }
                        Err(e) => {
                            eprintln!("Failed to open input file: {}", e);
                            continue;
                        }
                    }
                }

                if let Some((outfile, append)) = output_redirect {
                    let file = if append {
                        fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(outfile)
                    } else {
                        fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(outfile)
                    };

                    match file {
                        Ok(f) => {
                            command.stdout(Stdio::from(f));
                        }
                        Err(e) => {
                            eprintln!("Failed to open output file: {}", e);
                            continue;
                        }
                    }
                }

                match command.spawn() {
                    Ok(mut child) => {
                        let _ = child.wait();
                    }
                    Err(e) => {
                        eprintln!("Failed to execute command {}: {}", cmd, e);
                    }
                }
            }
        }
    }

    if let Some(ref path) = history_path {
        if let Err(e) = rl.save_history(path) {
            eprintln!("Warning: could not save history: {}", e);
        }
    }
}
