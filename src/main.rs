#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, fs, iter, os::unix::process::CommandExt, path::{Path, PathBuf}, process::{Command, Stdio}};

enum CommandType {
    Echo { text: Vec<String> },
    Exit { exit_code: i32 },
    Type { arg: String },
    Custom { cmd: String, args: Vec<String> },
    Pwd,
    Cd { path: String },
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
        // input.push(' ');
        // println!("'{input}'");
        // let mut iter = input.trim().split(' ');
        // let command = iter.next();

        let command = match input.split_once(char::is_whitespace) {
            None => continue,
            Some((cmd, args_str)) => {
                let mut quote = None;
                let mut preserve_next = false;
                let mut args_vec: Vec<String> = vec![];
                let mut last_str = String::new();

                args_str.chars().for_each(|c| {
                    if preserve_next {
                        last_str.push(c);
                        preserve_next = false;
                    } else if is_quote(c) && quote.is_some_and(|x| x == c) {
                        quote = None;
                    } else if is_quote(c) && quote.is_none() {
                        quote = Some(c);
                    } else if c == '\\' && quote.is_none() {
                        preserve_next = true;
                    } else if quote.is_some() || !c.is_whitespace() {
                        last_str.push(c);
                    } else if !last_str.is_empty() {
                        args_vec.push(last_str.clone());
                        last_str = String::new();
                    }
                });

                if !last_str.is_empty() {
                    args_vec.push(last_str);
                }
                // println!("{} {} - {:?}", cmd, args_str, args_vec);

                create_command(
                    cmd.to_string(),
                    args_vec,
                )
            },
        };

        match command {
            CommandType::Exit { exit_code  } => exit(exit_code),
            CommandType::Echo { text } => echo(&text),
            CommandType::Type { arg } => type_cmd(&arg),
            CommandType::Custom { cmd, args } => custom_cmd(&cmd, &args),
            CommandType::Pwd => pwd(),
            CommandType::Cd { path } => cd(&path),
        }
    }
}

fn pwd() {
    println!("{}", env::current_dir().unwrap().to_str().unwrap());
}

fn is_quote(c: char) -> bool {
    c == '\'' || c == '"'
}

fn cd(path_str: &str) {
    let path = if path_str == "~" { PathBuf::from(env::var("HOME").unwrap()) } else { PathBuf::from(path_str) };

    if env::set_current_dir(path).is_err() {
        println!("cd: {}: No such file or directory", path_str);
    }
}

fn custom_cmd(cmd: &str, args: &[String]) {
    let res = Command::new(cmd)
        .args(args)
        .stdout(Stdio::inherit())
        .spawn();

    match res {
        Ok(mut child) => {
            child.wait().unwrap();
        },
        Err(_) => {
            println!("{}: command not found", {cmd});
        }
    }

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
            CommandType::Custom { cmd: _, args: _ } => { 
                if let Some(dir) = check_path_for(cmd) {
                    println!("{} is {}/{}", cmd, dir, cmd)
                } else {
                    println!("{cmd}: not found");
                }
            },
            _ => println!("{cmd} is a shell builtin"),
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

fn create_command(name: String, args: Vec<String>) -> CommandType {
    match name.as_str() {
        "echo" => CommandType::Echo { text: args },
        "exit" => CommandType::Exit {
            exit_code: 0,
        },
        "type" => CommandType::Type {
            arg: args.first().map(String::from).unwrap_or(String::from("")),
        },
        "pwd" => CommandType::Pwd,
        "cd" => CommandType::Cd { path: args.first().map(String::from).unwrap_or(String::from("")) },
        cmd => CommandType::Custom {
            cmd: cmd.to_string(),
            args: args.iter().map(String::from).collect(),
        },
    }
}
