# building_my_own_minimalistic_shell

This project is a minimalistic shell built in Rust. It provides basic features like command execution, built-in commands, history support, and rudimentary completion.

## Features
- Parsing arguments with support for single and double quotes
- Built-in commands: `cd`, `pwd`, `echo`, `exit`, and `type`
- Basic command execution with input/output redirection (`<`, `>`, `>>`)
- Command history saved to `.rusty_shell_history` in the user's home directory
- Basic command completion for built-ins and files in the current directory

## Dependencies

Add these dependencies to your project using cargo:

```bash
cargo add rustyline
cargo add dirs
cargo run

