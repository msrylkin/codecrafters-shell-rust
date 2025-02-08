#[allow(unused_imports)]
use std::io::{self, Write};
use crossterm::{event::{self, read, Event, KeyCode, KeyEvent, KeyModifiers}, style::Print, terminal, ExecutableCommand};
use std::{env, fs::{self, File, OpenOptions}, io::{BufWriter, Read}, iter, os::{fd::FromRawFd, unix::process::CommandExt}, path::{Path, PathBuf}, process::{self, Command, Stdio}};

struct PathCmd(String, String);

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
        // io::stdout().execute(command)

        // Wait for user input
        // let mut stdin = io::stdin();

        // let mut input = String::new();
        // let mut buf: Vec<u8> = vec![0; 3];
        // stdin.read_exact(&mut buf).unwrap();

        // println!("{} {:?}", input, buf);
        let mut input = String::new();

        terminal::enable_raw_mode().unwrap();

        loop {
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => exit(0),
                    KeyCode::Char('j') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                        io::stdout().execute(Print("\r\n")).unwrap();
                        break;
                    },
                    KeyCode::Char(c) => {
                        io::stdout().execute(Print(c)).unwrap();
                        input.push(c);
                    },
                    KeyCode::Enter => {
                        // input.push('\n');
                        io::stdout().execute(Print("\r\n")).unwrap();
                        break;
                    },
                    KeyCode::Tab => {
                        match input.as_str() {
                            "ech" => {
                                io::stdout().execute(Print("o ")).unwrap();
                                input.push_str("o ");
                            },
                            "exi" => {
                                io::stdout().execute(Print("t ")).unwrap();
                                input.push_str("t ");
                            },
                            cmd => {
                                if let Some(PathCmd(full_cmd, ..)) = check_path_for_predicate(|x| x.starts_with(cmd)) {
                                    // dbg!(&full_cmd[cmd.len()..]);
                                    // dbg!(full_cmd);
                                    // io::stdout().execute(Print(&full_cmd)).unwrap();
                                    let rest = &full_cmd[cmd.len()..];
                                    // let rest = cmd;
                                    let rest_str = rest.to_string() + " ";
                                    io::stdout().execute(Print(&rest_str)).unwrap();
                                    input.push_str(rest_str.as_str());
                                } else {
                                    io::stdout().execute(Print("\x07")).unwrap();
                                }
                            },
                        }
                    }
                    _ => {}
                }
            }
        }

        terminal::disable_raw_mode().unwrap();
        // println!("res input - \"{}\"", input);

        let mut args_state = ArgsState::new();
        let mut char_handler = CharHandler::Unqouted;

        input.chars().for_each(|c| {
            char_handler = process_char(c, char_handler.clone(), &mut args_state);
        });

        let args_vec = args_state.finish();
        let mut args_vec_iter = args_vec.iter();

        let command = args_vec_iter.next();
        let command_args: Vec<String> = args_vec_iter.map(|x| x.to_string()).collect();

        match args_vec.last_chunk::<2>() {
            Some([redir, redir_file])  => {
                match redir.as_str() {
                    "2>>" => try_exec_command(
                        command,
                        &command_args[..command_args.len() - 2],
                        io::stdout(),
                        Box::new(
                            OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(redir_file)
                                .unwrap()
                        ) as Box<dyn Write>,
                    ),
                    ">>" | "1>>" => try_exec_command(
                        command,
                        &command_args[..command_args.len() - 2],
                        Box::new(
                                OpenOptions::new()
                                    .create(true)
                                    .append(true)
                                    .open(redir_file)
                                    .unwrap()
                        ) as Box<dyn Write>,
                        io::stderr(),
                    ),
                    "2>" => try_exec_command(
                        command,
                        &command_args[..command_args.len() - 2],
                        io::stdout(),
                        Box::new(File::create(redir_file).unwrap()) as Box<dyn Write>,
                    ),
                    "1>" | ">" => try_exec_command(
                        command,
                        &command_args[..command_args.len() - 2],
                        Box::new(File::create(redir_file).unwrap()) as Box<dyn Write>,
                        io::stderr(),
                    ),
                    _ => try_exec_command(
                        command,
                        &command_args,
                        io::stdout(),
                        io::stderr(),
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

fn try_exec_command(
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

fn check_path_for_start_with(cmd_beginning: &str) {

}

fn check_path_for_predicate<T: FnMut(&str) -> bool>(
    mut predicate: T,
) -> Option<PathCmd> {
    match env::var("PATH") {
        Ok(path) => path
            .split(':')
            .find_map(|dir| {
                if let Some(found_cmd) = check_dir_for_cmd_predicate(dir, &mut predicate) {
                    Some(PathCmd(found_cmd, dir.to_string()))
                } else {
                    None
                }
            }),
        Err(_) => None,
    }
}

fn check_path_for(cmd: &str) -> Option<String> {
    match env::var("PATH") {
        Ok(path) => path
            .split(':')
            .find(|dir| {
                check_dir_for_cmd_predicate(dir, |x| x == cmd).is_some()
            })
            .map(|dir| dir.to_string()),
        Err(_) => None,
    }
}

fn check_dir_for_cmd_predicate<T: FnMut(&str) -> bool>(
    dir: &str,
    // cmd: &str,
    mut predicate: T
) -> Option<String> {
    let dir = fs::read_dir(dir);

    if let Ok(dir) = dir {
        for path  in dir {
            if let Ok(path_item) = path {
                if let Some(last) = path_item.path().iter().last() {
                    if predicate(last.to_str().unwrap()) {
                        // println!("cmd: {}", last.to_string_lossy().to_string());
                        return Some(last.to_string_lossy().to_string());
                    }
                } 
            }
        }
    }

    None
}

fn exit(code: i32) {
    terminal::disable_raw_mode().unwrap();
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
