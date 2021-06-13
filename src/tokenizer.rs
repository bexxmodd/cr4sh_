pub struct Tokenizer {
    current: Option<String>,
}

impl Tokenizer {
    pub fn new(line: &str) -> Self {
        Tokenizer {
            current: Some(line.to_string()),
        }
    }
}

impl Iterator for Tokenizer {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = &mut self.current {
            let mut split: Vec<_> = s.split(' ').collect();
            let nxt = split.remove(0).to_string();
            if split.is_empty() {
                self.current = None;
            } else {
                self.current = Some(split.join(" "));
            }
            if nxt.is_empty() {
                None
            } else {
                Some(nxt)
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        let mut line = Tokenizer::new(&"");
        assert_eq!(None, line.next());
        assert_eq!(None, line.current);
    }

    #[test]
    fn two_word_string() {
        let mut line = Tokenizer::new(&"Hello World");
        assert_eq!("Hello".to_string(), line.next().unwrap());
        assert_eq!("World".to_string(), line.current.unwrap());
    }

    #[test]
    fn test_multiple_calls() {
        let mut line = Tokenizer::new(&"Hello Darkness My Old Friend");
        
        assert_eq!("Hello".to_string(), line.next().unwrap());
        assert_eq!("Darkness".to_string(), line.next().unwrap());
        assert_eq!("My".to_string(), line.next().unwrap());
        assert_eq!("Old".to_string(), line.next().unwrap());
        assert_eq!("Friend".to_string(), line.next().unwrap());
        assert_eq!(None, line.next());
    }
}