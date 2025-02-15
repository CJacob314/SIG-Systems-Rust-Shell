mod lexer;
mod parser;
mod safe_wrappers;

#[cfg(test)]
mod tests;

use std::io::{self, Write};

use parser::Command;

fn main() {
    // Input REPL
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
