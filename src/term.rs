
use std::{cmp, collections::HashMap, env, fmt::Display, io};

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
                                        fill_remaining_command(
                                            full_cmd.strip_prefix(cmd).unwrap(),
                                            &mut input,
                                        );
                                    } else {
                                        // let mut longest_common_pefix: Option<String> = None;

                                        // let mut all_suggestions = pathcmds
                                        //     .iter()
                                        //     .map(|e| e.command.clone())
                                        //     .collect::<Vec<String>>();
                                        // all_suggestions.sort();
                                        // let all_suggestions = all_suggestions;

                                        // for suggestion in all_suggestions.clone() {
                                        //     if suggestion.starts_with(cmd) {
                                        //         longest_common_pefix = match longest_common_pefix {
                                        //             Some(prefix) => {
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
                                        let mut longest_common_pefix = find_longest_common_fill(&pathcmds, cmd);

                                        if longest_common_pefix.clone().is_some_and(|x| x.len() > cmd.len()) {
                                            let  new_input = &longest_common_pefix.unwrap()[cmd.len()..];
                                            io::stdout().execute(Print(new_input)).unwrap();
                                            input.push_str(new_input);
                                            continue;
                                        }
                                        print_bell();

                                        if let Event::Key(event) = read().unwrap() {
                                            match event.code {
                                                KeyCode::Tab => {
                                                    let mut all_suggestions = pathcmds
                                                        .iter()
                                                        .map(|e| e.command.clone())
                                                        .collect::<Vec<String>>();
                                                    // all_suggestions.sort();
                                                    let all_suggestions = all_suggestions;
                                                    let all_suggestions_string = all_suggestions.join("  ");
                                                    print(format!("\r\n{all_suggestions_string}\r\n$ {cmd}"));
                                                },
                                                KeyCode::Char(c) => {
                                                    print(c);
                                                    input.push(c);
                                                },
                                                _ => {},
                                            }
                                        }
                                    }
                                } else {
                                    print_bell();
                                }
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
    env::var("PATH")
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
        .unwrap_or_default()
}

fn print<T: Display>(data: T) {
    io::stdout().execute(Print(data)).unwrap();
}

fn print_bell() {
    print("\x07");
}

fn pint_and_push(data: &str, target_string: &mut String) {
    print(data);
    target_string.push_str(data);
}

fn fill_remaining_command(
    postfix: &str,
    target_string: &mut String
) {
    let rest_str = format!("{} ", postfix);
    pint_and_push(&rest_str, target_string);
}

fn find_longest_common_fill(
    commands: &[PathCmd],
    prefix: &str
) -> Option<String> {
    // let mut longest_common_pefix: Option<String> = None;

    // let mut all_suggestions = commands
    //     .iter()
    //     .map(|e| e.command.clone())
    //     .collect::<Vec<String>>();
    // all_suggestions.sort();
    // let all_suggestions = all_suggestions;

    commands
        .iter()
        // .map(|e| e.command)
        .fold(<Option<&str>>::None, |longest_common, path_cmd| {
            if path_cmd.command.starts_with(prefix) {
                let retv = match longest_common {
                    None => {
                        return Some(&path_cmd.command);
                    },
                    Some(longest_common) => {
                        // let max = if longest_common.len() > path_cmd.command.len() { path_cmd.command.len() } else { longest_common.len() };
                        // let rightmost_i = cmp::min(longest_common.len(), path_cmd.command.len());
                        let (longer, shorter) = match path_cmd.command.chars().count().cmp(&longest_common.chars().count()) {
                            cmp::Ordering::Less => (longest_common, path_cmd.command.as_str()),
                            cmp::Ordering::Equal | cmp::Ordering::Greater => (path_cmd.command.as_str(), longest_common),
                        };
                        // let i = 0;
                        // let mut res = String::new();
                        let i = longer
                            .chars()
                            .zip(shorter.chars())
                            .take_while(|&(c_a, c_b)| c_a == c_b)
                            .count();

                        Some(&shorter[..i])
                        // for i in 0..max {
                        //     let prefix_i_char = longest_common.chars().nth(i).unwrap();
                        //     if  prefix_i_char != path_cmd.command.chars().nth(i).unwrap() {
                        //         break;
                        //     }

                        //     res.push(prefix_i_char);
                        // }

                        // return Some(&res);
                    }
                };

                return retv
            }

            longest_common
        })
        .map(|e| e.to_string())
        

    // for suggestion in all_suggestions.clone() {
    //     if suggestion.starts_with(prefix) {
    //         longest_common_pefix = match longest_common_pefix {
    //             Some(prefix) => {
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

    // longest_common_pefix
}