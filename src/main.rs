#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, fs::{self, File}, io::BufWriter, iter, os::{fd::FromRawFd, unix::process::CommandExt}, path::{Path, PathBuf}, process::{Command, Stdio}};

enum CommandType {
    Echo { text: Vec<String> },
    Exit { exit_code: i32 },
    Type { arg: String },
    Custom { cmd: String, args: Vec<String> },
    Pwd,
    Cd { path: String },
}

#[derive(Clone)]
enum CharHandler {
    SingleQoute,
    DoubleQoute,
    Unqouted,
    Preserve(Box<CharHandler>),
}

struct ArgsState {
    args: Vec<String>,
    res_string: String,
}

impl ArgsState {
    fn new() -> Self {
        Self {
            args: vec![],
            res_string: String::new(),
        }
    }

    fn push_char(&mut self, c: char) {
        self.res_string.push(c);
    }

    fn finish(mut self) -> Vec<String> {
        self.flush();

        self.args
    }

    fn flush(&mut self) {
        if !self.res_string.is_empty() {
            self.args.push(self.res_string.clone());
            self.res_string = String::new();
        }
    }
}

fn process_char(
    c: char,
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match c {
        '\\' => process_backslash(handler, state),
        '\'' => process_single_qoute(handler, state),
        '"' => process_double_qoute(handler, state),
        // '$' => process_dollar(handler, state),
        c if c.is_whitespace() => process_whitespace(c, handler, state),
        c => process_symbol(c, handler, state),
    }
}

fn process_backslash(
    handler: CharHandler,
    state: &mut ArgsState
) -> CharHandler {
    match handler {
        CharHandler::Unqouted | CharHandler::DoubleQoute => CharHandler::Preserve(Box::new(handler)),
        CharHandler::Preserve(parent_handler) => {
            state.push_char('\\');

            *parent_handler
        },
        _ => {
            state.push_char('\\');

            handler
        }
    }
}

fn process_single_qoute(
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match handler {
        CharHandler::DoubleQoute => {
            state.push_char('\'');

            CharHandler::DoubleQoute
        },
        CharHandler::SingleQoute => CharHandler::Unqouted,
        CharHandler::Unqouted => CharHandler::SingleQoute,
        CharHandler::Preserve(inner_handler) => {
            match *inner_handler {
                CharHandler::DoubleQoute => {
                    state.push_char('\\');
                    state.push_char('\'');
                },
                _ => {
                    state.push_char('\'');
                } 
            };

            *inner_handler
        },
    }
}

fn process_symbol(
    c: char,
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match handler {
        CharHandler::Preserve(inner_handler) => {
            match *inner_handler {
                CharHandler::DoubleQoute => {
                    state.push_char('\\');
                    state.push_char(c);
                },
                _ => {
                    state.push_char(c);
                } 
            };

            *inner_handler
        },
        _ => {
            state.push_char(c);

            handler
        }
    }
}

fn process_double_qoute(
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match handler {
        CharHandler::Preserve(inner_handler) => {
            state.push_char('"');

            *inner_handler
        },
        CharHandler::SingleQoute => {
            state.push_char('"');

            handler
        },
        CharHandler::DoubleQoute => {
            CharHandler::Unqouted
        },
        CharHandler::Unqouted => {
            CharHandler::DoubleQoute
        }
    }
}

fn process_whitespace(
    c: char,
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match handler {
        CharHandler::Preserve(inner_handler) => {
            state.push_char(c);

            *inner_handler
        },
        CharHandler::SingleQoute | CharHandler::DoubleQoute => {
            state.push_char(c);

            handler
        },
        CharHandler::Unqouted => {
            state.flush();

            handler
        }
    }
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

        let mut args_state = ArgsState::new();
        let mut char_handler = CharHandler::Unqouted;

        input.chars().for_each(|c| {
            char_handler = process_char(c, char_handler.clone(), &mut args_state);
        });

        let args_vec = args_state.finish();
        let mut args_vec_iter = args_vec.iter();

        let command = args_vec_iter.next();
        let command_args: Vec<String> = args_vec_iter.map(|x| x.to_string()).collect();
        // let iostd = io::stdout();
        // let mut out_writer: Box<dyn Write> = BufWriter::new(inner)

        match args_vec.last_chunk::<2>() {
            Some([redir, redir_file])  => {
                match redir.as_str() {
                    "1>" | ">" => try_exec_command(
                        command,
                        &command_args[..command_args.len() - 2],
                        // Stdio::from(File::create(redir_file).unwrap())
                        Box::new(File::create(redir_file).unwrap()) as Box<dyn Write>,
                    ),
                    _ => try_exec_command(
                        command,
                        &command_args,
                        // Stdio::inherit(),
                        io::stdout(),
                    ),
                }
            },
            None => try_exec_command(
                command,
                &command_args,
                // Stdio::inherit(),
                io::stdout(),
            ),  
        };

        // let command = match command {
        //     None => continue,
        //     Some(cmd) => create_command(cmd.clone(), args_vec_iter.map(|x| x.to_string()).collect())
        // };

        // let command = match input.split_once(char::is_whitespace) {
        //     None => continue,
        //     Some((cmd, args_str)) => {
        //         let mut args_state = ArgsState::new();
        //         let mut char_handler = CharHandler::Unqouted;

        //         args_str.chars().for_each(|c| {
        //             char_handler = process_char(c, char_handler.clone(), &mut args_state);
        //         });

        //         let args_vec = args_state.finish();
        //         // println!("res args: {args_vec:?}");

        //         create_command(
        //             cmd.to_string(),
        //             args_vec,
        //         )
        //     },
        // };

        // match command {
        //     CommandType::Exit { exit_code  } => exit(exit_code),
        //     CommandType::Echo { text } => echo(&text),
        //     CommandType::Type { arg } => type_cmd(&arg),
        //     CommandType::Custom { cmd, args } => custom_cmd(&cmd, &args),
        //     CommandType::Pwd => pwd(),
        //     CommandType::Cd { path } => cd(&path),
        // }
    }
}

fn try_exec_command(
    command: Option<&String>,
    args: &[String],
    out: impl Write,
) {
    // let mut args_vec_iter = args.iter();
    let command = match command {
        None => return,
        // Some(cmd) => create_command(cmd.clone(), args_vec_iter.map(|x| x.to_string()).collect())
        Some(cmd) => create_command(cmd.to_string(), args)
    };

    match command {
        CommandType::Exit { exit_code  } => exit(exit_code),
        CommandType::Echo { text } => echo(&text, out),
        CommandType::Type { arg } => type_cmd(&arg),
        CommandType::Custom { cmd, args } => custom_cmd(&cmd, &args, out),
        CommandType::Pwd => pwd(),
        CommandType::Cd { path } => cd(&path),
    };
}

fn pwd() {
    println!("{}", env::current_dir().unwrap().to_str().unwrap());
}

fn is_quote(c: char) -> bool {
    c == '\'' || c == '"'
}

fn is_escapable(c: char) -> bool {
    c == '\\' || c == '$' || c == '"'
}

fn cd(path_str: &str) {
    let path = if path_str == "~" { PathBuf::from(env::var("HOME").unwrap()) } else { PathBuf::from(path_str) };

    if env::set_current_dir(path).is_err() {
        println!("cd: {}: No such file or directory", path_str);
    }
}

fn custom_cmd(cmd: &str, args: &[String], mut out: impl Write) {
    let res = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .spawn();

    match res {
        Ok(child) => {
            let output = child.wait_with_output().unwrap();
            out.write_all(&output.stdout).unwrap();
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

fn echo(args: &[String], mut out: impl Write) {
    // println!("{}", args.join(" "));
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
