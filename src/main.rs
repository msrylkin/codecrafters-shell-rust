#[allow(unused_imports)]
use std::io::{self, Write};
use crossterm::{event::{self, read, Event, KeyCode, KeyEvent, KeyModifiers}, style::Print, terminal, ExecutableCommand};
use std::{env, fs::{self, File, OpenOptions}, io::{BufWriter, Read}, iter, os::{fd::FromRawFd, unix::process::CommandExt}, path::{Path, PathBuf}, process::{self, Command, Stdio}};

mod command;
mod lib;
mod args;
mod term;

use command::*;
use lib::*;
use args::*;
use term::*;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let input = Term::new(|| exit(0)).read();

        let args_vec = ArgsParser::new(input).parse();
        let mut args_vec_iter = args_vec.iter();
        let command = args_vec_iter.next();
        let command_args: Vec<String> = args_vec_iter.map(|x| x.to_string()).collect();

        match args_vec.last_chunk::<2>() {
            Some([redir, redir_file])  => {
                match redir.as_str() {
                    "2>>" => try_exec_command(
                        command,
                        &command_args[..command_args.len() - 2],
                        default_stdout(),
                        build_append_file_pipe(redir_file),
                    ),
                    ">>" | "1>>" => try_exec_command(
                        command,
                        &command_args[..command_args.len() - 2],
                        build_append_file_pipe(redir_file),
                        default_stderr(),
                    ),
                    "2>" => try_exec_command(
                        command,
                        &command_args[..command_args.len() - 2],
                        default_stdout(),
                        build_file_pipe(redir_file),
                    ),
                    "1>" | ">" => try_exec_command(
                        command,
                        &command_args[..command_args.len() - 2],
                        build_file_pipe(redir_file),
                        default_stderr(),
                    ),
                    _ => try_exec_command(
                        command,
                        &command_args,
                        default_stdout(),
                        default_stderr(),
                    ),
                }
            },
            None => try_exec_command(
                command,
                &command_args,
                io::stdout(),
                io::stderr(),
            ),  
        };
    }
}

fn build_append_file_pipe(file: &str) -> Box<dyn Write> {
    Box::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(file)
            .unwrap()
    ) as Box<dyn Write>
}

fn build_file_pipe(file: &str) -> Box<dyn Write> {
    Box::new(File::create(file).unwrap()) as Box<dyn Write>
}

fn default_stdout() -> impl Write {
    io::stdout()
}

fn default_stderr() -> impl Write {
    io::stderr()
}