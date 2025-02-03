#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

fn main() {
    loop {
        // Uncomment this block to pass the first stage
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let command = input.trim();

        if command.starts_with("exit") {
            break;
        }

        match command {
            _ => println!("{command}: command not found"),
        };
    }
}
