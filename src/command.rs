use std::fs;
use std::process::{Command, Stdio};
use std::{env, path::PathBuf};
use std::io::{self, Write};

use crate::env_util::*;

pub enum CommandType {
    Echo { text: Vec<String> },
    Exit { exit_code: i32 },
    Type { arg: String },
    Custom { cmd: String, args: Vec<String> },
    Pwd,
    Cd { path: String },
}

pub fn try_exec_command(
    command: Option<&String>,
    args: &[String],
    out: impl Write,
    err_out: impl Write,
) {
    let command = match command {
        None => return,
        Some(cmd) => create_command(cmd.to_string(), args)
    };

    match command {
        CommandType::Exit { exit_code  } => exit(exit_code),
        CommandType::Echo { text } => echo(&text, out),
        CommandType::Type { arg } => type_cmd(&arg),
        CommandType::Custom { cmd, args } => custom_cmd(&cmd, &args, out, err_out),
        CommandType::Pwd => pwd(),
        CommandType::Cd { path } => cd(&path),
    };
}

fn create_command(name: String, args: &[String]) -> CommandType {
    match name.as_str() {
        "echo" => CommandType::Echo { text: args.iter().map(|x| x.to_string()).collect() },
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

fn pwd() {
    println!("{}", env::current_dir().unwrap().to_str().unwrap());
}

fn cd(path_str: &str) {
    let path = if path_str == "~" { PathBuf::from(env::var("HOME").unwrap()) } else { PathBuf::from(path_str) };

    if env::set_current_dir(path).is_err() {
        println!("cd: {}: No such file or directory", path_str);
    }
}

fn echo(args: &[String], mut out: impl Write) {
    writeln!(out, "{}", args.join(" ")).unwrap();
}

fn type_cmd(cmd: &str) {
    if !cmd.is_empty() {
        match create_command(cmd.to_string(), &[]) {
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

fn custom_cmd(cmd: &str, args: &[String], mut out: impl Write, mut err_out: impl Write) {
    let res = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match res {
        Ok(child) => {
            let output = child.wait_with_output().unwrap();
            out.write_all(&output.stdout).unwrap();
            err_out.write_all(&output.stderr).unwrap();
        },
        Err(_) => {
            writeln!(err_out, "{}: command not found", {cmd}).unwrap();
        }
    }
}

pub fn exit(code: i32) {
    std::process::exit(code);
}

fn check_path_for(cmd: &str) -> Option<String> {
    match env::var("PATH") {
        Ok(path) => path
            .split(':')
            .find(|dir| {
                !check_dir_for_cmd_predicate(dir, |x| x == cmd).is_empty()
            })
            .map(|dir| dir.to_string()),
        Err(_) => None,
    }
}
