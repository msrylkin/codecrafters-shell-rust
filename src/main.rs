#[allow(unused_imports)]
use std::io::{self, Write};

enum CommandType {
    Echo { text: Vec<String> },
    Exit { exit_code: i32 },
    Type { arg: String },
}

// struct Command {
//     name: String,
//     args: Vec<String>,
// }

// trait Executable {
//     fn execute(args: &Vec<String>);
// }

// impl Executable for Command {
    
// }

// enum Command {
//     Echo { name: String, args: Vec<String> },
// }

fn main() {
    loop {
        // Uncomment this block to pass the first stage
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let mut iter = input.trim().split(' ');
        let command = iter.next();

        // if command.is_none() {
        //     continue;
        // }

        // let command = Command {
        //     name: command.unwrap().to_string(),
        //     args: iter.map(|s| s.to_string()).collect(),
        // };

        // match command.name.as_str() {
        //     "echo" => echo(&iter.collect()),
        //     "exit" => std::process::exit(0),
        //     "type" => type_cmd(&iter.collect()),
        //     cmd => println!("{cmd}: command not found"),
        // }

        let command_object = match command {
            None => continue,
            Some(cmd) => create_command(
                cmd.to_string(),
                iter.map(String::from).collect(),
            )
        };

        if let Some(command) = command_object {
            match command {
                CommandType::Exit { exit_code  } => exit(exit_code),
                CommandType::Echo { text } => echo(&text),
                CommandType::Type { arg } => type_cmd(&arg),
            }
        } else {
            println!("{}: command not found", {command.unwrap()});
        }
        // if command_object.is_none() {
        //     println!("{}: command not found", {command.unwrap()});
        //     return;
        // }

        // let command = match command {
        //     None => continue,
        //     // Some("echo") => echo(&iter.collect()),
        //     Some("echo") => CommandType::Echo { text: iter.map(String::from).collect() },
        //     // Some("exit") => std::process::exit(0),
        //     Some("exit") => CommandType::Exit {
        //         exit_code: 0,
        //     },
        //     Some("type") => CommandType::Type {
        //         arg: iter.next().unwrap_or_default().to_string(),
        //     },
        //     Some(cmd) => println!("{cmd}: command not found"),
        // };
    }
}

fn echo(args: &[String]) {
    println!("{}", args.join(" "));
}

fn type_cmd(cmd: &str) {
    if !cmd.is_empty() {
        match create_command(cmd.to_string(), vec![]) {
            Some(_) => println!("{cmd} is a shell builtin"),
            None => println!("{cmd}: not found"), 
        }
    }
}

fn exit(code: i32) {
    std::process::exit(code);
}

fn create_command(name: String, args: Vec<String>) -> Option<CommandType> {
    match name.as_str() {
        // Some("echo") => echo(&iter.collect()),
        "echo" => Some(CommandType::Echo { text: args }),
        // Some("exit") => std::process::exit(0),
        "exit" => Some(CommandType::Exit {
            exit_code: 0,
        }),
        "type" => Some(CommandType::Type {
            arg: args.first().map(String::from).unwrap_or(String::from("")),
        }),
        // cmd => println!("{cmd}: command not found"),
        _ => None,
    }
}
