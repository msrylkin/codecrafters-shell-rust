#[allow(unused_imports)]
use std::io::{self, Write};
use crossterm::{event::{self, read, Event, KeyCode, KeyEvent, KeyModifiers}, style::Print, terminal, ExecutableCommand};
use std::{env, fs::{self, File, OpenOptions}, io::{BufWriter, Read}, iter, os::{fd::FromRawFd, unix::process::CommandExt}, path::{Path, PathBuf}, process::{self, Command, Stdio}};

mod command;
mod lib;
mod args;

use command::*;
use lib::*;
use args::*;

#[derive(Hash, Eq, PartialEq, Clone)]
struct PathCmd(String, String);

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
                                if let Some(pathcmds) = check_path_for_predicate(|x| x.starts_with(cmd)) {
                                    if !pathcmds.is_empty() {
                                        if pathcmds.len() == 1 {
                                            let full_cmd = &pathcmds[0].0;
                                            let rest = &full_cmd[cmd.len()..];
                                            let rest_str = rest.to_string() + " ";
                                            io::stdout().execute(Print(&rest_str)).unwrap();
                                            input.push_str(rest_str.as_str());
                                        } else {
                                            let mut longest_common_pefix: Option<String> = None;

                                            let mut all_suggestions = pathcmds
                                                            .iter()
                                                            .map(|e| e.0.clone())
                                                            .collect::<Vec<String>>();
                                                        all_suggestions.sort();
                                                        let all_suggestions = all_suggestions;

                                            for suggestion in all_suggestions.clone() {
                                                if suggestion.starts_with(cmd) {
                                                    longest_common_pefix = match longest_common_pefix {
                                                        Some(prefix) => {
                                                            // dbg!(prefix.clone(), suggestion.clone());
                                                            // Some(prefix)
                                                            let max = if prefix.len() > suggestion.len() { suggestion.len() } else { prefix.len() };
                                                            let mut res = String::new();
                                                            for i in 0..max {
                                                                let prefix_i_char = prefix.chars().nth(i).unwrap();
                                                                if  prefix_i_char != suggestion.chars().nth(i).unwrap() {
                                                                    break;
                                                                }

                                                                res.push(prefix_i_char);
                                                            }

                                                            Some(res)
                                                        },
                                                        None => Some(suggestion),
                                                    }
                                                }
                                            }

                                            if longest_common_pefix.clone().is_some_and(|x| x.len() > cmd.len()) {
                                                let  new_input = &longest_common_pefix.unwrap()[cmd.len()..];
                                                io::stdout().execute(Print(new_input)).unwrap();
                                                input.push_str(new_input);
                                                continue;
                                            }
                                            io::stdout().execute(Print("\x07")).unwrap();

                                            if let Event::Key(event) = read().unwrap() {
                                                match event.code {
                                                    KeyCode::Tab => {
                                                        let mut all_suggestions = pathcmds
                                                            .iter()
                                                            .map(|e| e.0.clone())
                                                            .collect::<Vec<String>>();
                                                        all_suggestions.sort();
                                                        let all_suggestions = all_suggestions;

                                                        // let mut longest_common_pefix: Option<String> = None;

                                                        // for suggestion in all_suggestions.clone() {
                                                        //     if suggestion.starts_with(cmd) {
                                                        //         longest_common_pefix = match longest_common_pefix {
                                                        //             Some(prefix) => {
                                                        //                 // dbg!(prefix.clone(), suggestion.clone());
                                                        //                 // Some(prefix)
                                                        //                 let max = if prefix.len() > suggestion.len() { suggestion.len() } else { prefix.len() };
                                                        //                 let mut res = String::new();
                                                        //                 for i in 0..max {
                                                        //                     let prefix_i_char = prefix.chars().nth(i).unwrap();
                                                        //                     if  prefix_i_char != suggestion.chars().nth(i).unwrap() {
                                                        //                         break;
                                                        //                     }

                                                        //                     res.push(prefix_i_char);
                                                        //                 }

                                                        //                 Some(res)
                                                        //             },
                                                        //             None => Some(suggestion),
                                                        //         }
                                                        //     }
                                                        // }


                                                        // dbg!(
                                                        //     longest_common_pefix.clone(),
                                                        //     cmd.clone(),
                                                        //     longest_common_pefix.clone().unwrap().len(),
                                                        //     cmd.clone().len(),
                                                        // );
                                                        // if longest_common_pefix.clone().is_some_and(|x| x.len() > cmd.len()) {
                                                            // let  new_input = &longest_common_pefix.unwrap()[cmd.len()..];
                                                            // io::stdout().execute(Print(new_input)).unwrap();
                                                            // input.push_str(new_input);
                                                        // } else {
                                                        // io::stdout().execute(Print(format!(": res - {longest_common_pefix:?}"))).unwrap();
                                                            let all_suggestions_string = all_suggestions.join("  ");
                                                            io::stdout().execute(Print(format!("\r\n{all_suggestions_string}\r\n$ {cmd}"))).unwrap();
                                                        // }
                                                        // io::stdout().execute(Print(format!("\r\n{all_suggestions_string}"))).unwrap();
                                                    },
                                                    KeyCode::Char(c) => {
                                                        io::stdout().execute(Print(c)).unwrap();
                                                        input.push(c);
                                                    },
                                                    _ => {},
                                                }
                                            }
                                        }
                                    } else {
                                        io::stdout().execute(Print("\x07")).unwrap();
                                    }
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

fn check_path_for_predicate<T: FnMut(&str) -> bool>(
    mut predicate: T,
) -> Option<Vec<PathCmd>> {
    match env::var("PATH") {
        Ok(path) => {
            let mut res_vec: Vec<PathCmd> = vec![];

            
            path
                .split(':')
                .for_each(|dir| {
                    let cmds = check_dir_for_cmd_predicate(dir, &mut predicate);
                    cmds
                        .iter()
                        .for_each(|cmd| {
                            if res_vec.iter().find(|z| &z.0 == cmd).is_none() {
                                res_vec.push(PathCmd(cmd.to_string(), path.to_string()));
                            }
                        })
                });
            
            Some(res_vec)
        },
        Err(_) => None,
    }
}

