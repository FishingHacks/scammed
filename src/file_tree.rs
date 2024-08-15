use core::str;
use std::{os::unix::ffi::OsStringExt, path::{Path, PathBuf}};

use anathema::state::{List, State, Value};

static BLACKLIST: &[&'static str] = &["target", ".git"];

#[derive(State, Debug)]
pub struct Folder {
    files: Value<List<String>>,
    folders: Value<List<String>>,
}

pub fn empty_folder() -> Folder {
    Folder { files: List::empty(), folders: List::empty() }
}

pub fn read_file_tree(path: &Path, currently_focused: &Path) -> Folder {
    let Ok(contents) = path.read_dir() else {
        return empty_folder();
    };
    
    let mut folder = empty_folder();
    for f in contents {
        let Ok(f) = f else {
            continue;
        };
        if currently_focused.starts_with(&f.path()) {
            println!("currently_focused blocking {}", f.path().display());
            continue;
        }
        println!("{}", f.path().display());
        let Ok(name) = String::from_utf8(f.file_name().into_vec()) else { panic!("failed to convert {:?} to utf-8", f.file_name()); };
        if BLACKLIST.contains(&name.as_str()) {
            continue;
        }
        let Ok(typ) = f.file_type() else { continue; };
        if typ.is_dir() {
            folder.folders.push_back(name);
        } else if typ.is_file() {
            folder.files.push_back(name);
        }
    }

    folder
}

pub fn get_path_list(path: &Path, currently_focused: PathBuf) -> Value<List<String>> {
    let mut list = List::empty();

    let Ok(path) = currently_focused.strip_prefix(path) else { return list; };
    
    for (index, path_section) in path.iter().enumerate() {
        if let Ok(name) = str::from_utf8(path_section.as_encoded_bytes()) {
            let mut padded_name = " ".repeat(index * 2);
            padded_name.push_str(name);
            list.push_back(padded_name);
        }
    }


    list
}
