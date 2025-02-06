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

#[derive(Clone)]
enum CharHandler {
    SingleQouteHandler,
    DoubleQouteHandler,
    UnqoutedHandler,
    PreserveHandler(Box<CharHandler>),
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

// fn process_dollar(
//     handler: CharHandler,
//     state: &mut ArgsState
// ) -> CharHandler {
//     match handler {
//         CharHandler::PreserveHandler(parent_handler) => {
//             match parent_handler {
                
//             }
//         },
//     }
// }

fn process_backslash(
    handler: CharHandler,
    state: &mut ArgsState
) -> CharHandler {
    match handler {
        CharHandler::UnqoutedHandler | CharHandler::DoubleQouteHandler => CharHandler::PreserveHandler(Box::new(handler)),
        CharHandler::PreserveHandler(parent_handler) => {
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
        CharHandler::DoubleQouteHandler => {
            state.push_char('\'');

            CharHandler::DoubleQouteHandler
        },
        CharHandler::SingleQouteHandler => CharHandler::UnqoutedHandler,
        CharHandler::UnqoutedHandler => CharHandler::SingleQouteHandler,
        // CharHandler::PreserveHandler(inner_handler) => {
        //     state.push_char('\'');

        //     *inner_handler
        // },
        CharHandler::PreserveHandler(inner_handler) => {
            match *inner_handler {
                CharHandler::DoubleQouteHandler => {
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
        // CharHandler::DoubleQouteHandler => {
        //     state.push_char(c);

        //     CharHandler::DoubleQouteHandler
        // },
        // CharHandler::SingleQouteHandler => {
        //     state.push_char(c);

        //     CharHandler::SingleQouteHandler
        // },
        // CharHandler::UnqoutedHandler => {
        //     state.push_char(c);

        //     CharHandler::UnqoutedHandler
        // },
        CharHandler::PreserveHandler(inner_handler) => {
            match *inner_handler {
                CharHandler::DoubleQouteHandler => {
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
    // state.push_char(c);
}

fn process_double_qoute(
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match handler {
        CharHandler::PreserveHandler(inner_handler) => {
            state.push_char('"');

            *inner_handler
        },
        CharHandler::SingleQouteHandler => {
            state.push_char('"');

            handler
        },
        CharHandler::DoubleQouteHandler => {
            CharHandler::UnqoutedHandler
        },
        CharHandler::UnqoutedHandler => {
            CharHandler::DoubleQouteHandler
        }
    }
}

fn process_whitespace(
    c: char,
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match handler {
        CharHandler::PreserveHandler(inner_handler) => {
            state.push_char(c);

            *inner_handler
        },
        CharHandler::SingleQouteHandler => {
            state.push_char(c);

            handler
        },
        CharHandler::DoubleQouteHandler => {
            state.push_char(c);

            handler
        },
        CharHandler::UnqoutedHandler => {
            state.flush();

            handler
        }
    }
}


// struct SingleQouteHandler;
// struct DoubleQouteHandler;
// struct UnqoutedHandler;


// trait CharHandler {
//     fn handle_single_qoute(
//         self: Box<Self>,
//         c: char,
//         current_string: &mut String,
//         args: &mut Vec<String>,
//     ) -> Box<dyn CharHandler>;
//     fn handle_double_qoute(
//         self: Box<Self>,
//         c: char,
//         current_string: &mut String,
//         args: &mut Vec<String>,
//     ) -> Box<dyn CharHandler>;
//     fn handle_whitespace(
//         self: Box<Self>,
//         c: char,
//         current_string: &mut String,
//         args: &mut Vec<String>,
//     ) -> Box<dyn CharHandler>;
//     fn handle_symbol(
//         self: Box<Self>,
//         c: char,
//         current_string: &mut String,
//         args: &mut Vec<String>,
//     ) -> Box<dyn CharHandler>;
    
//     fn handle(
//         self: Box<Self>,
//         c: char,
//         current_string: &mut String,
//         args: &mut Vec<String>,
//     ) -> Box<dyn CharHandler> {
//         match c {
//             '\'' => self.handle_single_qoute(c, current_string, args),
//             '"' => self.handle_double_qoute(c, current_string, args),
//             c if c.is_whitespace() => self.handle_whitespace(c, current_string, args),
//             c => self.handle_symbol(c, current_string, args),
//         }
//     }
// }

// impl CharHandler for UnqoutedHandler {
//     fn handle_single_qoute(
//         self: Box<Self>,
//         c: char,
//         current_string: &mut String,
//         args: &mut Vec<String>,
//     ) -> Box<dyn CharHandler> {
//         // processor.push_char(c);
//         current_string.push(c);

//         self
//     }

//     fn handle_whitespace(self: Box<Self>, c: char, current_string: &mut String, args: &mut Vec<String>) -> Box<dyn CharHandler> {
//         // string.push(c);
//         // processor.flush_current_string();
//         if !current_string.is_empty() {
//             args.push(current_string.clone());
//             *current_string = String::new();
//         }

//         self
//     }

//     fn handle_double_qoute(self: Box<Self>, c: char, current_string: &mut String, args: &mut Vec<String>) -> Box<dyn CharHandler> {
//         // processor.push_char(c);
//         current_string.push(c);

//         self
//     }

//     fn handle_symbol(self: Box<Self>, c: char, current_string: &mut String, args: &mut Vec<String>) -> Box<dyn CharHandler> {
//         // processor.push_char(c);
//         current_string.push(c);

//         self
//     }
// }

// struct ArgsState {
//     args: Vec<String>,
//     current_string: String,
// }

// struct CharProcessor {
//     handler: Box<dyn CharHandler>,
//     args: Vec<String>,
//     current_string: String,
// }

// impl CharProcessor {
//     fn new() -> Self {
//         CharProcessor {
//             handler: Box::new(UnqoutedHandler),
//             args: vec![],
//             current_string: String::new(),
//         }
//     }

//     fn process(&mut self, c: char) {
//         self.handler = self.handler.handle(c, &mut self.current_string, &mut self.args);
//         // self.handler.handle(c, self);
//     }

//     fn finish(mut self) -> Vec<String> {
//         self.flush_current_string();

//         self.args
//     }

//     fn flush_current_string(&mut self) {
//         if !self.current_string.is_empty() {
//             self.args.push(self.current_string.clone());
//             self.current_string = String::new();
//         }
//     }

//     fn push_char(&mut self, c: char) {
//         self.current_string.push(c);
//     } 
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
        // input.push(' ');
        // println!("'{input}'");
        // let mut iter = input.trim().split(' ');
        // let command = iter.next();

        let command = match input.split_once(char::is_whitespace) {
            None => continue,
            Some((cmd, args_str)) => {
                // let mut quote = None;
                // let mut preserve_next = false;
                // let mut custom_preserve = false;
                // let mut args_vec: Vec<String> = vec![];
                // let mut last_str = String::new();
                // let mut charProcessor = CharProcessor::new();
                let mut args_state = ArgsState::new();
                let mut char_handler = CharHandler::UnqoutedHandler;

                args_str.chars().for_each(|c| {
                    char_handler = process_char(c, char_handler.clone(), &mut args_state);
                    // match c {
                    //     '\'' | '"' => {
                    //         quote = Some(c);
                    //     },
                    //     c if c.is_whitespace() && quote.is_some() => {},
                    //     c => {},
                    // };
                    // if preserve_next {
                    //     if quote.is_some_and(|x| x == '"') && is_escapable(c) {
                    //         last_str.push('\\');
                    //     }
                    //     last_str.push(c);
                    //     preserve_next = false;
                    //     custom_preserve = false;
                    // } else if is_quote(c) && quote.is_some_and(|x| x == c) {
                    //     quote = None;
                    // } else if is_quote(c) && quote.is_none() {
                    //     quote = Some(c);
                    // } else if c == '\\' {
                    //     preserve_next = true;
                    //     if quote.is_some_and(|x| x == '"') {
                    //         custom_preserve = true;
                    //     }
                    // } else if quote.is_some() || !c.is_whitespace() {
                    //     last_str.push(c);
                    // } else if !last_str.is_empty() {
                    //     args_vec.push(last_str.clone());
                    //     last_str = String::new();
                    // }

                    // // println!("{:?} {} {} {} |{}|", quote, preserve_next, custom_preserve, c, last_str);
                });

                // if !last_str.is_empty() {
                //     args_vec.push(last_str);
                // }
                // println!("{} {} - {:?}", cmd, args_str, args_vec);
                // let args_vec = charProcessor.finish();
                let args_vec = args_state.finish();
                // println!("res args: {args_vec:?}");

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

fn is_escapable(c: char) -> bool {
    c == '\\' || c == '$' || c == '"'
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
