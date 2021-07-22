use fs_set_times::{set_mtime, SystemTimeSpec};
use std::fs;
use std::io::Result;
use std::path::Path;
use std::thread;
use std::time::SystemTime;

fn touch(option: char, file: &str) -> Result<()> {
    if Path::new(file).exists() {
        let now = SystemTime::now();
        set_mtime(file, SystemTimeSpec::from(now))?;
    } else {
        fs::File::create(file)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use core::time;

    use super::*;

    #[test]
    fn create_file() {
        let _ = touch('a', "test001.txt");
        assert!(Path::new("test.txt").exists());
        if let Err(_) = fs::remove_file("test.txt") {
            eprintln!("Can't remove test.txt");
        }
    }

    #[test]
    fn test_updated_timestamp() {
        let _ = touch('a', "test002.txt");
        let mut metadata = fs::metadata("test002.txt").unwrap();
        let init_time = metadata.modified().unwrap();

        thread::sleep(time::Duration::from_secs(3));

        let _ = touch('a', "test002.txt");
        metadata = fs::metadata("test002.txt").unwrap();
        let modified_time = metadata.modified().unwrap();

        let diff = modified_time.duration_since(init_time).unwrap().as_secs();
        assert_eq!(diff, time::Duration::from_secs(3).as_secs());

        if let Err(_) = fs::remove_file("test002.txt") {
            eprintln!("Can't remove test.txt");
        }
    }
}
