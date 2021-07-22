use fs_set_times::{set_atime, set_mtime, SystemTimeSpec};
use std::fs;
use std::io::Result;
use std::path::Path;
use std::thread;
use std::time::SystemTime;

pub fn touch(option: Option<String>, file: &str) -> Result<()> {
    let mut flag = true;
    let mut refer = None;
    if let Some(op) = option {
        for o in op.chars().into_iter() {
            match o {
                'c' => flag = false,
                'a' => set_time(file, refer.clone(), flag, set_atime)?,
                'm' => set_time(file, refer.clone(), flag, set_mtime)?,
                _ => eprintln!("{} is invalid operand", o),
            }
        }
    } else {
        set_time(file, refer.clone(), flag, set_atime)?;
        set_time(file, refer.clone(), flag, set_mtime)?;
    }
    Ok(())
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
    use core::time;

    use super::*;

    #[test]
    fn create_file() {
        let _ = touch(None, "test001.txt");
        assert!(Path::new("test001.txt").exists());
        if let Err(_) = fs::remove_file("test001.txt") {
            eprintln!("Can't remove test.txt");
        }
    }

    #[test]
    fn test_updated_modification() {
        let _ = touch(None, "test002.txt");
        let mut metadata = fs::metadata("test002.txt").unwrap();
        let init_time = metadata.modified().unwrap();

        thread::sleep(time::Duration::from_secs(1));

        let _ = set_time("test002.txt", None, false, set_mtime);
        metadata = fs::metadata("test002.txt").unwrap();
        let modified_time = metadata.modified().unwrap();

        let diff = modified_time.duration_since(init_time).unwrap().as_secs();
        assert_eq!(diff, time::Duration::from_secs(1).as_secs());

        if let Err(_) = fs::remove_file("test002.txt") {
            eprintln!("Can't remove test.txt");
        }
    }

    #[test]
    fn test_updated_access() {
        let _ = touch(None, "test003.txt");
        let mut metadata = fs::metadata("test003.txt").unwrap();
        let init_time = metadata.accessed().unwrap();

        thread::sleep(time::Duration::from_secs(1));

        let _ = set_time("test003.txt", None, false, set_atime);
        metadata = fs::metadata("test003.txt").unwrap();
        let modified_time = metadata.accessed().unwrap();

        let diff = modified_time.duration_since(init_time).unwrap().as_secs();
        assert_eq!(diff, time::Duration::from_secs(1).as_secs());

        if let Err(_) = fs::remove_file("test003.txt") {
            eprintln!("Can't remove test.txt");
        }
    }
}
