use std::io::{self, Write};

mod command;
mod args;
mod term;
mod out;
mod env_util;

use command::*;
use args::*;
use term::*;
use out::*;

pub fn run() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let input = Term::new(|| exit(0)).read();

        let args_vec = ArgsParser::new(input).parse();
        let mut args_vec_iter = args_vec.iter();
        let command = args_vec_iter.next();
        let command_args: Vec<String> = args_vec_iter.map(|x| x.to_string()).collect();

        match command_args.split_last_chunk::<2>() {
            Some((rest_args, [redir, redir_file])) => {
                match get_pipes(&(redir, redir_file)) {
                    Some(pipes) => {
                        try_exec_command(command, rest_args, pipes.stdout_target, pipes.stderr_target);
                    },
                    None => {
                        let pipes = get_default_pipes();

                        try_exec_command(command, &command_args, pipes.stdout_target, pipes.stderr_target);
                    }
                };
            },
            None => {
                let pipes: OutPipes = get_default_pipes();

                try_exec_command(command, &command_args, pipes.stdout_target, pipes.stderr_target);
            }
        }
    }
}


