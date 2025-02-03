#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        // Uncomment this block to pass the first stage
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let mut iter = input.split_whitespace();
        let command = iter.next();

        match command {
            None => continue,
            Some("echo") => echo(&iter.collect()),
            Some("exit") => std::process::exit(0), 
            Some(cmd) => println!("{cmd}: command not found"),
        };
    }
}

fn echo(args: &Vec<&str>) {
    println!("{}", args.join(" "));
}