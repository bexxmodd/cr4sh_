use crate::Tokenizer;
use fs_set_times::{set_atime, set_mtime, SystemTimeSpec};
use std::io::{ErrorKind, Result};
use std::path::Path;
use std::time::SystemTime;
use std::{fs, io};

pub fn touch(tokenizer: &mut Tokenizer) -> Result<()> {
    let cmd = parse_command(tokenizer).unwrap();
    let mut create_flag = true;

    let mut newfile_index = 0usize;
    let mut reffile_index = 0usize;
    let mut options_index = 0usize;
    for (i, val) in cmd.iter().enumerate() {
        if val.starts_with("-") {
            options_index = i;
            if cmd[options_index].contains("r") {
                if cmd.len() > i + 1 {
                    reffile_index = i + 1;
                } else {
                    return Err(io::Error::new(
                        ErrorKind::InvalidInput,
                         "Invalid argument"));
                }
                if cmd.len() > i + 2 {
                    newfile_index = i + 2;
                } else {
                    newfile_index = i - 1
                }
            } else {
                newfile_index = i + 1;
            }
        }
    }

    // will need when 'r' flag is implemented
    let refer = if reffile_index > 0 {
        get_reference_timestamp(&cmd[reffile_index])
    } else {
        None
    };
    if cmd.len() >= 2 {
        for op in cmd[options_index].chars().into_iter() {
            match op {
                'c' => create_flag = false,
                'a' => set_time(&cmd[newfile_index], &refer, create_flag, set_atime)?,
                'm' => set_time(&cmd[newfile_index], &refer, create_flag, set_mtime)?,
                'r' => set_time(&cmd[newfile_index], &refer, create_flag, set_mtime)?,
                '-' => continue,
                _ => eprintln!("{} is invalid operand", op),
            }
        }
    } else {
        set_time(&cmd[newfile_index], &refer, create_flag, set_atime)?;
        set_time(&cmd[newfile_index], &refer, create_flag, set_mtime)?;
    }
    Ok(())
}

fn parse_command(tokenizer: &mut Tokenizer) -> Result<Vec<String>> {
    tokenizer.next();
    let res: Vec<_> = tokenizer.collect();
    Ok(res)
}

fn set_time(
    src: &str,
    refer: &Option<SystemTime>,
    flag: bool,
    func: fn(path: String, atime: SystemTimeSpec) -> Result<()>,
) -> Result<()> {
    let time = if refer.is_some() {
        refer.unwrap()
    } else {
        SystemTime::now()
    };

    if !Path::new(src).exists() && flag {
        fs::File::create(src)?;
    } 
    func(src.to_string(), SystemTimeSpec::from(time))?;
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
        let mut token = Tokenizer::new("touch test000.txt");
        let _ = touch(&mut token);

        assert!(Path::new(filename).exists());

        if let Err(_) = fs::remove_file(filename) {
            eprintln!("Can't remove {}", filename);
        }
    }

    #[test]
    fn test_no_file_creation() {
        let filename = "test001.txt";
        let mut token = Tokenizer::new("touch -c test001.txt");
        let _ = touch(&mut token);
        assert!(!Path::new(filename).exists());
    }

    #[test]
    fn test_updated_modification() {
        let filename = "test002.txt";
        let mut token = Tokenizer::new("touch test002.txt");
        let _ = touch(&mut token);
        let mut metadata = fs::metadata(filename).unwrap();
        let init_time = metadata.modified().unwrap();

        sleep(time::Duration::from_secs(1));

        let _ = set_time(filename, &None, false, set_mtime);
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
        let mut token = Tokenizer::new("touch test003.txt");
        let _ = touch(&mut token);
        let mut metadata = fs::metadata(filename).unwrap();
        let init_time = metadata.accessed().unwrap();

        sleep(time::Duration::from_secs(1));

        let _ = set_time(filename, &None, false, set_atime);
        metadata = fs::metadata(filename).unwrap();
        let modified_time = metadata.accessed().unwrap();

        let diff = modified_time.duration_since(init_time).unwrap().as_secs();
        assert_eq!(diff, time::Duration::from_secs(1).as_secs());

        if let Err(_) = fs::remove_file(filename) {
            eprintln!("Can't remove {}", filename);
        }
    }

    #[test]
    fn test_set_reffile_time() {
        let filename = "test004.txt";
        let mut del_token = Tokenizer::new("touch delme");
        let _ = touch(&mut del_token);

        let metadata = fs::metadata("delme").unwrap();

        sleep(time::Duration::from_secs(1));

        let mut token = Tokenizer::new("touch -r delme test004.txt");
        let _ = touch(&mut token);
        let result_metadata = fs::metadata(filename).unwrap();

        assert_eq!(metadata.modified().unwrap(), result_metadata.modified().unwrap());

        if let Err(_) = fs::remove_file(filename) {
            eprintln!("Can't remove {}", filename);
        }

        if let Err(_) = fs::remove_file("delme") {
            eprintln!("Can't remove delme");
        } 
    }
}
