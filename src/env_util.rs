use std::{collections::HashMap, env, fs};

pub struct PathCmd {
    pub command: String,
    pub path: String,
}

pub fn check_path_for(cmd: &str) -> Option<String> {
    match env::var("PATH") {
        Ok(path) => path
            .split(':')
            .find(|dir| {
                !check_dir_for_cmd_predicate(dir, |x| x == cmd).is_empty()
            })
            .map(|dir| dir.to_string()),
        Err(_) => None,
    }
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

pub fn check_path_for_predicate<T: FnMut(&str) -> bool>(
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