use std::{cmp, collections::HashMap, env, fmt::Display, io};

use crossterm::{event::{read, Event, KeyCode, KeyEvent, KeyModifiers}, style::Print, terminal};
use crossterm::ExecutableCommand;
use crate::lib::*;

pub struct Term<F: Fn()> {
    on_exit: F,
}

struct PathCmd {
    command: String,
    path: String,
}

enum TermSignal {
    Exit,
    EndLine,
    EndChar,
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
                match process_key_event(event, &mut input) {
                    TermSignal::Exit => (self.on_exit)(),
                    TermSignal::EndChar => {}, // continue loop
                    TermSignal::EndLine => break,
                }
            }
        }

        terminal::disable_raw_mode().unwrap();

        input
    }
}

fn process_key_event(event: KeyEvent, input: &mut String) -> TermSignal {
    match event.code {
        KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => TermSignal::Exit,
        KeyCode::Char('j') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            print("\r\n");

            TermSignal::EndLine
        },
        KeyCode::Char(c) => {
            print_and_push(&String::from(c), input);

            TermSignal::EndChar
        },
        KeyCode::Enter => {
            print_and_push("\r\n", input);

            TermSignal::EndLine
        },
        KeyCode::Tab => handle_tab(input),
        _ => TermSignal::EndChar,
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

fn print_and_push(data: &str, target_string: &mut String) {
    print(data);
    target_string.push_str(data);
}

fn fill_remaining_command(
    postfix: &str,
    target_string: &mut String
) {
    let rest_str = format!("{} ", postfix);
    print_and_push(&rest_str, target_string);
}

fn find_longest_common_prefix(
    commands: &[PathCmd],
) -> Option<&str> {
    commands
        .iter()
        .map(|PathCmd {command, ..}| command.as_str())
        .reduce(|current_common_prefix, command| {
            let (longer, shorter) = match command.chars().count().cmp(&current_common_prefix.chars().count()) {
                cmp::Ordering::Less => (current_common_prefix, command),
                cmp::Ordering::Equal | cmp::Ordering::Greater => (command, current_common_prefix),
            };

            let i = longer
                .chars()
                .zip(shorter.chars())
                .take_while(|&(c_a, c_b)| c_a == c_b)
                .count();

            &shorter[..i]
        })
}

fn handle_tab(input: &mut String) -> TermSignal {
    match input.as_str() {
        "ech" => {
            io::stdout().execute(Print("o ")).unwrap();
            input.push_str("o ");

            TermSignal::EndChar
        },
        "exi" => {
            io::stdout().execute(Print("t ")).unwrap();
            input.push_str("t ");

            TermSignal::EndChar
        },
        _ => process_autocomplete(input),
    }
}

fn process_autocomplete(input: &mut String) -> TermSignal {
    let path_commands = check_path_for_predicate(|x| x.starts_with(input.as_str()));

    match path_commands.len() {
        0 => { 
            print_bell();
            
            TermSignal::EndChar
        },
        1 => {
            let full_cmd = &path_commands[0].command;
            fill_remaining_command(
                full_cmd.strip_prefix(input.as_str()).unwrap(),
                input,
            );

            TermSignal::EndChar
        },
        _ => {
            match find_longest_common_prefix(&path_commands) {
                Some(longest_common_prefix) if longest_common_prefix.len() > input.len() => {
                    print_and_push(&longest_common_prefix[input.len()..], input);

                    TermSignal::EndChar
                },
                None | Some(_) => {
                    print_bell();

                    match read().unwrap() {
                        Event::Key(event) => match event.code {
                            KeyCode::Tab => {
                                let all_suggestions = path_commands
                                    .into_iter()
                                    .map(|e| e.command)
                                    .collect::<Vec<_>>();
                                let all_suggestions_string = all_suggestions.join("  ");
                                print(format!("\r\n{all_suggestions_string}\r\n$ {input}"));

                                TermSignal::EndChar
                            },
                            _ => process_key_event(event, input),
                        },
                        _ => TermSignal::EndChar,
                    }
                },
            }
        }
    }
}