mod lexer;
mod parser;

#[cfg(test)]
mod tests;

use libc::{execle, execve, fork};
use std::io::{self, Write};

use parser::Command;

fn main() {
    // Input REPL loop
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" {
            break;
        }

        let command = Command::parse(input);
        println!("{command:#?}");
    }
}
