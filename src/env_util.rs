use std::{collections::HashMap, env, fs};

pub struct PathCmd {
    pub command: String,
    pub path: String,
}

pub fn get_first_path_command(cmd: &str) -> Option<String> {
    env::var("PATH")
        .ok()
        .and_then(|path_env_val|
            get_path_commands_iter(&path_env_val, |e| e == cmd)
                .take(1)
                .next()
                .map(|e| e.path)
        )
}

pub fn check_dir_for_cmd_predicate<T: FnMut(&str) -> bool>(
    dir: &str,
    mut predicate: T
) -> Vec<String> {
    fs::read_dir(dir)
        .ok()
        .map(|read_dir| {
            read_dir.filter_map(|dir_entry| {
                dir_entry.ok().and_then(|ok_dir_entry| {
                    ok_dir_entry
                        .file_name()
                        .to_str()
                        .map(String::from)
                        .filter(|str| predicate(str))
                })
            })
            .collect()
        })
        .unwrap_or_default()
}

pub fn get_path_commands<T: FnMut(&str) -> bool>(
    predicate: T,
) -> Vec<PathCmd> {
    try_get_path_commands(predicate).unwrap_or_default()
}

fn try_get_path_commands<T: FnMut(&str) -> bool>(
    predicate: T,
) -> Option<Vec<PathCmd>> {
    env::var("PATH")
        .ok()
        .map(|path_env| {
            let mut commands_vec = get_path_commands_iter(&path_env, predicate)
                .map(|path_cmd| (path_cmd.command.to_string(), path_cmd))
                .collect::<HashMap<_, _>>()
                .into_values()
                .collect::<Vec<PathCmd>>();

            commands_vec.sort_by(|a, b| a.command.cmp(&b.command));

            commands_vec
        })
}

fn get_path_commands_iter<'a, T: FnMut(&str) -> bool + 'a>(
    path_env: &'a str,
    mut predicate: T,
) -> impl Iterator<Item = PathCmd> + 'a {
    path_env
        .split(':')
        .flat_map(move |dir| check_dir_for_cmd_predicate(dir, &mut predicate).into_iter().map(|command| PathCmd {
            command,
            path: dir.to_string(),
        }))
        
}