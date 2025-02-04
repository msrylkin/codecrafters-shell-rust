#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, fs, os::unix::process::CommandExt, process::{Command, Stdio}};

enum CommandType {
    Echo { text: Vec<String> },
    Exit { exit_code: i32 },
    Type { arg: String },
    Custom { cmd: String, args: Vec<String> }
}

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
                CommandType::Custom { cmd, args } => custom_cmd(&cmd, &args),
            }
        } else {
            println!("{}: command not found", {command.unwrap()});
        }
    }
}

fn custom_cmd(cmd: &str, args: &[String]) {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::inherit())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    // println!("{}", res);

    // Command::new(cmd)
    //     .args(args)
    //     .exec();
}

fn echo(args: &[String]) {
    println!("{}", args.join(" "));
}

fn type_cmd(cmd: &str) {
    if !cmd.is_empty() {
        match create_command(cmd.to_string(), vec![]) {
            Some(_) => println!("{cmd} is a shell builtin"),
            None => { 
                if let Some(dir) = check_path_for(cmd) {
                    println!("{} is {}/{}", cmd, dir, cmd)
                } else {
                    println!("{cmd}: not found");
                }
            }, 
        }
    }
}

fn check_path_for(cmd: &str) -> Option<String> {
    match env::var("PATH") {
        Ok(path) => path
            .split(':')
            .find(|dir| {
                check_dir_for_cmd(dir, cmd).unwrap_or(false)
            })
            .map(|dir| dir.to_string()),
        Err(_) => None,
    }
}

fn check_dir_for_cmd(dir: &str, cmd: &str) -> Result<bool, std::io::Error> {
    let dir = fs::read_dir(dir)?;
    // println!("{:?}", dir);

    for path  in dir {
        if let Ok(path_item) = path {
            if let Some(last) = path_item.path().iter().last() {
                if last == cmd {
                    return Ok(true);
                }
            }
        } 
    }

    Ok(false)
}

fn exit(code: i32) {
    std::process::exit(code);
}

fn create_command(name: String, args: Vec<String>) -> Option<CommandType> {
    match name.as_str() {
        "echo" => Some(CommandType::Echo { text: args }),
        "exit" => Some(CommandType::Exit {
            exit_code: 0,
        }),
        "type" => Some(CommandType::Type {
            arg: args.first().map(String::from).unwrap_or(String::from("")),
        }),
        cmd => {
            Some(CommandType::Custom {
                cmd: cmd.to_string(),
                args: args.iter().map(String::from).collect(),
            })
        },
    }
}
