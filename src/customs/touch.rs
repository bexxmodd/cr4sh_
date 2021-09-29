use crate::Tokenizer;
use fs_set_times::{set_atime, set_mtime, SystemTimeSpec};
use std::collections::HashSet;
use std::io::{ErrorKind, Result};
use std::path::Path;
use std::time::SystemTime;
use std::{fs, io};

pub fn touch(tokenizer: &mut Tokenizer) -> Result<()> {
    let cmd = parse_command(tokenizer).unwrap();
    let mut create_flag = true;

    let mut newfile_index = 0usize;
    let mut reffile_index = 0usize;
    let mut options_index = usize::MAX;
    for (i, val) in cmd.iter().enumerate() {
        if val.starts_with('-') {
            options_index = i;
            if cmd[options_index].contains('r') {
                if cmd.len() > i + 1 {
                    reffile_index = i + 1;
                } else {
                    return Err(io::Error::new(
                                ErrorKind::InvalidInput, "Invalid argument"
                            ));
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

    if options_index < cmd.len() {
        for op in cmd[options_index].chars().into_iter() {
            match op {
                '-' => continue,
                'c' => create_flag = false,
                'a' => set_time(&cmd[newfile_index..], &refer,
                                create_flag, set_atime)?,
                'm' => set_time(&cmd[newfile_index..], &refer,
                                create_flag, set_mtime)?,
                'r' => set_time(&cmd[newfile_index..], &refer,
                                create_flag, set_mtime)?,
                _ => eprintln!("{} is invalid operand", op),
            }
        }
    } else {
        set_time(&cmd[newfile_index..], &refer, create_flag, set_atime)?;
        set_time(&cmd[newfile_index..], &refer, create_flag, set_mtime)?;
    }
    Ok(())
}

fn parse_command(tokenizer: &mut Tokenizer) -> Result<Vec<String>> {
    tokenizer.next();
    let symbols: HashSet<_> = 
        vec!["~", "#", "@", "<", ">", "&", "|", ">", "%", "*", "(", ")", "!"]
            .into_iter()
            .collect();
    let res: Vec<_> = tokenizer.filter(|v| !symbols.contains(&v[0..1])).collect();
    Ok(res)
}

fn set_time(
    src: &[String],
    refer: &Option<SystemTime>,
    flag: bool,
    func: fn(path: String, atime: SystemTimeSpec) -> Result<()>,
) -> Result<()> {
    let time = if refer.is_some() {
        refer.unwrap()
    } else {
        SystemTime::now()
    };

    for f in src {
        if !Path::new(f).exists() && flag {
            fs::File::create(f)?;
        }
        func(f.to_string(), SystemTimeSpec::from(time))?;
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
        let mut token = Tokenizer::new("touch test000.txt");
        let _ = touch(&mut token);

        assert!(Path::new(filename).exists());

        if fs::remove_file(filename).is_err() {
            eprintln!("Can't remove {}", filename);
        }
    }

    #[test]
    fn test_parser() {
        let mut line = Tokenizer::new("trulala -a nu 'patom paidzom'");
        let expected: Vec<String> = vec![
            "-a".to_string(), "nu".to_string(), "patom paidzom".to_string()
        ];
        assert_eq!(expected, parse_command(&mut line).unwrap());


    }

    #[test]
    fn test_create_with_space_in_name() {
        let filename1 = "test file";
        let filename2 = "second file";
        let mut token = Tokenizer::new("touch 'test file' \"second file\"");

        let _ = touch(&mut token);

        assert!(Path::new(filename1).exists());
        assert!(Path::new(filename2).exists());

        if fs::remove_file(filename1).is_err() {
            eprintln!("Can't remove {}", filename1);
        }

        if fs::remove_file(filename2).is_err() {
            eprintln!("Can't remove {}", filename2);
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
    fn test_create_multiple_files() {
        let files = [
            "multi01".to_string(),
            "multi02".to_string(),
            "multi03".to_string()
        ];
        let mut token = Tokenizer::new("touch multi01 multi02 multi03");
        let res = touch(&mut token);
        assert!(res.is_ok());

        for f in files.iter() {
            assert!(Path::new(f).exists());
        }

        for f in files.iter() {
            if fs::remove_file(f).is_err() {
                eprintln!("Can't remove {}", f);
            }
        }
    }

    #[test]
    fn test_updated_modification() {
        let filename = "test002.txt";
        let mut token = Tokenizer::new("touch test002.txt");
        let _ = touch(&mut token);
        let mut metadata = fs::metadata(filename).unwrap();
        let init_time = metadata.modified().unwrap();

        sleep(time::Duration::from_secs(1));

        let _ = set_time(&[filename.to_string()], &None, false, set_mtime);
        metadata = fs::metadata(filename).unwrap();
        let modified_time = metadata.modified().unwrap();

        let diff = modified_time.duration_since(init_time).unwrap().as_secs();
        assert_eq!(diff, time::Duration::from_secs(1).as_secs());

        if fs::remove_file(filename).is_err() {
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

        let _ = set_time(&[filename.to_string()], &None, false, set_atime);
        metadata = fs::metadata(filename).unwrap();
        let modified_time = metadata.accessed().unwrap();

        let diff = modified_time.duration_since(init_time).unwrap().as_secs();
        assert_eq!(diff, time::Duration::from_secs(1).as_secs());

        if fs::remove_file(filename).is_err() {
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

        assert_eq!(
            metadata.modified().unwrap(),
            result_metadata.modified().unwrap()
        );

        if fs::remove_file(filename).is_err() {
            eprintln!("Can't remove {}", filename);
        }

        if fs::remove_file("delme").is_err() {
            eprintln!("Can't remove delme");
        }
    }
}
