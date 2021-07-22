use crate::Tokenizer;
use fs_set_times::{set_atime, set_mtime, SystemTimeSpec};
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::time::SystemTime;

pub fn touch(tokenizer: &mut Tokenizer) -> Result<()> {
    let cmd = parse_command(tokenizer).unwrap();
    let mut flag = true;
    let mut refer = None; // will need when 'r' flag is implemented
    if cmd.len() == 2 {
        for op in cmd[1].chars().into_iter() {
            match op {
                'c' => flag = false,
                'a' => set_time(&cmd[0], refer.clone(), flag, set_atime)?,
                'm' => set_time(&cmd[0], refer.clone(), flag, set_mtime)?,
                _ => eprintln!("{} is invalid operand", op),
            }
        }
    } else {
        set_time(&cmd[0], refer.clone(), flag, set_atime)?;
        set_time(&cmd[0], refer.clone(), flag, set_mtime)?;
    }
    Ok(())
}

fn parse_command(tokenizer: &mut Tokenizer) -> Result<Vec<String>> {
    assert_eq!("touch".to_string(), tokenizer.next().unwrap());
    let mut res: Vec<_> = vec![];

    if tokenizer.starts_with("-") {
        res.push(tokenizer.next().unwrap());
    }
    res.push(tokenizer.next().unwrap());
    Ok(res)
}

fn set_time(
    src: &str,
    refer: Option<String>,
    flag: bool,
    func: fn(path: String, atime: SystemTimeSpec) -> Result<()>,
) -> Result<()> {
    if Path::new(src).exists() {
        let time = if refer.is_some() {
            get_reference_timestamp(&refer.unwrap()).unwrap()
        } else {
            SystemTime::now()
        };

        func(src.to_string(), SystemTimeSpec::from(time))?;
    } else if flag {
        fs::File::create(src)?;
    }
    Ok(())
}

fn get_reference_timestamp(refer: &str) -> Option<SystemTime> {
    if Path::new(refer).exists() {
        return Some(fs::metadata(refer).unwrap().modified().unwrap());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::time;
    use std::thread::sleep;

    #[test]
    fn test_create_file() {
        let filename = "test000.txt";
        let _ = touch(None, filename);

        assert!(Path::new(filename).exists());

        if let Err(_) = fs::remove_file(filename) {
            eprintln!("Can't remove {}", filename);
        }
    }

    #[test]
    fn test_no_file_creation() {
        let filename = "test001.txt";
        let _ = touch(Some("c".to_string()), filename);
        assert!(!Path::new(filename).exists());
    }

    #[test]
    fn test_updated_modification() {
        let filename = "test002.txt";
        let _ = touch(None, filename);
        let mut metadata = fs::metadata(filename).unwrap();
        let init_time = metadata.modified().unwrap();

        sleep(time::Duration::from_secs(1));

        let _ = set_time(filename, None, false, set_mtime);
        metadata = fs::metadata(filename).unwrap();
        let modified_time = metadata.modified().unwrap();

        let diff = modified_time.duration_since(init_time).unwrap().as_secs();
        assert_eq!(diff, time::Duration::from_secs(1).as_secs());

        if let Err(_) = fs::remove_file(filename) {
            eprintln!("Can't remove {}", filename);
        }
    }

    #[test]
    fn test_updated_access() {
        let filename = "test003.txt";
        let _ = touch(None, filename);
        let mut metadata = fs::metadata(filename).unwrap();
        let init_time = metadata.accessed().unwrap();

        sleep(time::Duration::from_secs(1));

        let _ = set_time(filename, None, false, set_atime);
        metadata = fs::metadata(filename).unwrap();
        let modified_time = metadata.accessed().unwrap();

        let diff = modified_time.duration_since(init_time).unwrap().as_secs();
        assert_eq!(diff, time::Duration::from_secs(1).as_secs());

        if let Err(_) = fs::remove_file(filename) {
            eprintln!("Can't remove {}", filename);
        }
    }
}
