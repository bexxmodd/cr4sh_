
use crate::{shellname::ShellName, tokenizer::Tokenizer};
use std::{
    env::{current_dir, set_current_dir},
    path::PathBuf,
};

/// Implementation of a Linux's `cd` command,
/// which stands for change directory.
pub fn change_directory(shell_name: &mut ShellName, line: &mut Tokenizer) {
    assert_eq!("cd".to_string(), line.next().unwrap());

    let path = line.next();

    let new_path: PathBuf = if path.is_some() {
        let tmp = path.unwrap();
        if tmp.eq("~") {
            dirs::home_dir().unwrap()
        } else {
            PathBuf::from(tmp)
        }
    } else {
        dirs::home_dir().unwrap()
    };

    if let Err(e) = set_current_dir(new_path) {
        eprintln!("Error: {}", e);
    } else {
        let cur = current_dir().unwrap();
        shell_name.set_current_dir(&cur.to_str().unwrap());
    }
}