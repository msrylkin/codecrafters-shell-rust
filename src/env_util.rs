use std::fs;

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