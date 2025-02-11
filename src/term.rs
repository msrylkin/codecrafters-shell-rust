
use std::{collections::HashMap, env, fmt::Display, io};

use crossterm::{event::{read, Event, KeyCode, KeyModifiers}, style::Print, terminal};
use crossterm::ExecutableCommand;
use crate::lib::*;

pub struct Term<F: Fn()> {
    on_exit: F,
}

struct PathCmd {
    command: String,
    path: String,
}


impl<F: Fn()> Term<F> {
    pub fn new(on_exit: F) -> Self {
        Term {
            on_exit,
        }
    }

    pub fn read(&self) -> String {
        let mut input = String::new();

        terminal::enable_raw_mode().unwrap();

        loop {
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                        (self.on_exit)();
                    },
                    KeyCode::Char('j') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                        print("\r\n");
                        break;
                    },
                    KeyCode::Char(c) => {
                        print(c);
                        input.push(c);
                    },
                    KeyCode::Enter => {
                        print("\r\n");
                        input.push_str("\r\n");
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
                                let pathcmds = check_path_for_predicate(|x| x.starts_with(cmd));
                                if !pathcmds.is_empty() {
                                    if pathcmds.len() == 1 {
                                        let full_cmd = &pathcmds[0].command;
                                        let rest = &full_cmd[cmd.len()..];
                                        let rest_str = rest.to_string() + " ";
                                        io::stdout().execute(Print(&rest_str)).unwrap();
                                        input.push_str(rest_str.as_str());
                                    } else {
                                        let mut longest_common_pefix: Option<String> = None;

                                        let mut all_suggestions = pathcmds
                                                        .iter()
                                                        .map(|e| e.command.clone())
                                                        .collect::<Vec<String>>();
                                        all_suggestions.sort();
                                        let all_suggestions = all_suggestions;

                                        for suggestion in all_suggestions.clone() {
                                            if suggestion.starts_with(cmd) {
                                                longest_common_pefix = match longest_common_pefix {
                                                    Some(prefix) => {
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
                                                        .map(|e| e.command.clone())
                                                        .collect::<Vec<String>>();
                                                    all_suggestions.sort();
                                                    let all_suggestions = all_suggestions;
                                                    let all_suggestions_string = all_suggestions.join("  ");
                                                    io::stdout().execute(Print(format!("\r\n{all_suggestions_string}\r\n$ {cmd}"))).unwrap();
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
                                // if let Some(pathcmds) = check_path_for_predicate(|x| x.starts_with(cmd)) {
                                    
                                // } else {
                                //     io::stdout().execute(Print("\x07")).unwrap();
                                // }
                            },
                        }
                    }
                    _ => {}
                }
            }
        }

        terminal::disable_raw_mode().unwrap();

        input
    }
}

fn check_path_for_predicate<T: FnMut(&str) -> bool>(
    mut predicate: T,
) -> Vec<PathCmd> {
    let res = env::var("PATH")
        .ok()
        .map(|path_env| {
            let mut commands_vec = path_env
                .split(':')
                .flat_map(|dir| {
                    check_dir_for_cmd_predicate(
                        dir,
                        &mut predicate
                    )
                        .into_iter()
                        .map(|command| 
                            (command.to_string(), PathCmd {
                                command: command.to_string(),
                                path: dir.to_string(),
                            })
                        )
                })
                .collect::<HashMap<_, _>>()
                .into_values()
                .collect::<Vec<PathCmd>>();
                
            commands_vec.sort_by(|a, b| a.command.cmp(&b.command));

            commands_vec
        })
        .unwrap_or_default();

    res
    // match env::var("PATH") {
    //     Ok(path) => {
    //         let mut res_vec: Vec<PathCmd> = vec![];
            
    //         path
    //             .split(':')
    //             .for_each(|dir| {
    //                 let cmds = check_dir_for_cmd_predicate(dir, &mut predicate);
    //                 cmds
    //                     .iter()
    //                     .for_each(|cmd| {
    //                         if res_vec.iter().find(|z| &z.0 == cmd).is_none() {
    //                             res_vec.push(PathCmd(cmd.to_string(), path.to_string()));
    //                         }
    //                     })
    //             });
            
    //         Some(res_vec)
    //     },
    //     Err(_) => None,
    // }
}

fn print<T: Display>(data: T) {
    io::stdout().execute(Print(data)).unwrap();
}