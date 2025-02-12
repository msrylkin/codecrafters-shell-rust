use std::{fs::{File, OpenOptions}, io::{self, Write}};

pub struct OutPipes {
    pub stdout_target: Box<dyn Write>,
    pub stderr_target: Box<dyn Write>,
}

pub fn get_pipes(&(redir, redir_file): &(&str, &str)) -> Option<OutPipes> {
    match redir {
        "2>>" => OutPipes { stdout_target: default_stdout(), stderr_target: build_append_file_pipe(redir_file) }.into(),
        ">>" | "1>>" => OutPipes { stdout_target: build_append_file_pipe(redir_file), stderr_target: default_stderr() }.into(),
        "2>" => OutPipes { stdout_target: default_stdout(), stderr_target: build_file_pipe(redir_file) }.into(),
        "1>" | ">" => OutPipes { stdout_target: build_file_pipe(redir_file), stderr_target: default_stderr() }.into(),
        _ => None
        // _ => OutPipes { stdout_target: default_stdout(), stderr_target: default_stderr() },
    }
    // match maybe_redir_params.last_chunk::<2>() {
    //     Some([redir, redir_file])  => {
    //         match redir.as_str() {
    //             "2>>" => try_exec_command(
    //                 command,
    //                 &command_args[..command_args.len() - 2],
    //                 default_stdout(),
    //                 build_append_file_pipe(redir_file),
    //             ),
    //             ">>" | "1>>" => try_exec_command(
    //                 command,
    //                 &command_args[..command_args.len() - 2],
    //                 build_append_file_pipe(redir_file),
    //                 default_stderr(),
    //             ),
    //             "2>" => try_exec_command(
    //                 command,
    //                 &command_args[..command_args.len() - 2],
    //                 default_stdout(),
    //                 build_file_pipe(redir_file),
    //             ),
    //             "1>" | ">" => try_exec_command(
    //                 command,
    //                 &command_args[..command_args.len() - 2],
    //                 build_file_pipe(redir_file),
    //                 default_stderr(),
    //             ),
    //             _ => try_exec_command(
    //                 command,
    //                 &command_args,
    //                 default_stdout(),
    //                 default_stderr(),
    //             ),
    //         }
    //     },
    //     None => try_exec_command(
    //         command,
    //         &command_args,
    //         io::stdout(),
    //         io::stderr(),
    //     ),  
    // };
}

pub fn get_default_pipes() -> OutPipes {
    OutPipes { stdout_target: default_stdout(), stderr_target: default_stderr() }
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

fn default_stdout() -> Box<dyn Write> {
    Box::new(io::stdout())
}

fn default_stderr() -> Box<dyn Write> {
    Box::new(io::stderr())
}