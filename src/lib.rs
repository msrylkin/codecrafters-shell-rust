use std::fs;

pub fn check_dir_for_cmd_predicate<T: FnMut(&str) -> bool>(
    dir: &str,
    // cmd: &str,
    mut predicate: T
) -> Vec<String> {
    let dir = fs::read_dir(dir);

    let mut vec: Vec<String> = vec![];

    if let Ok(dir) = dir {
        for path  in dir {
            if let Ok(path_item) = path {
                if let Some(last) = path_item.path().iter().last() {
                    if predicate(last.to_str().unwrap()) {
                        // println!("cmd: {}", last.to_string_lossy().to_string());
                        // return Some(last.to_string_lossy().to_string());
                        vec.push(last.to_string_lossy().to_string());
                    }
                } 
            }
        }
    }

    vec
}