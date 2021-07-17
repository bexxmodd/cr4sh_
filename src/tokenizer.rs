#[derive(Debug)]
pub struct Tokenizer {
    current: Option<String>,
}

impl Tokenizer {
    pub fn new(line: &str) -> Self {
        Tokenizer {
            current: Some(line.to_string()),
        }
    }

    pub fn args_before_redirection(&mut self) -> Vec<String> {
        let mut args = vec![];
        while self.current.is_some() {
            if self.peek().eq(">") || self.peek().eq("<") {
                break;
            } else {
                args.push(self.next().unwrap());
            }
        }
        args
    }

    pub fn commands_before_pipe(&mut self) -> Tokenizer {
        let mut before = String::new();
        while let Some(a) = self.next() {
            if a.eq("|") {
                break;
            }
            before.push_str(&a);
            before.push_str(" ");
        }
        Tokenizer::new(&before)
    }

    pub fn get_args(&mut self) -> Vec<String> {
        let mut args = vec![];
        while let Some(a) = self.next() {
            args.push(a);
        }
        args
    }

    pub fn is_pipe(&self) -> bool {
        self.contains("|")
    }

    pub fn contains(&self, pattern: &str) -> bool {
        if let Some(cur) = self.current.as_ref() {
            cur.contains(pattern)
        } else {
            false
        }
    }

    pub fn has_redirection(&self) -> bool {
        self.contains(">") || self.contains("<")
    }

    pub fn peek(&self) -> String {
        let mut res = "".to_string();
        if let Some(cur) = self.current.clone() {
            for c in cur.chars().into_iter() {
                if c.eq(&' ') {
                    break;
                } else {
                    res.push(c);
                }
            }
        }
        res
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

    #[test]
    fn test_peek() {
        let mut line = Tokenizer::new(&"Hello Darkness > My");
        assert_eq!("Hello".to_string(), line.peek());
        assert_eq!("Hello".to_string(), line.next().unwrap());
        assert_eq!("Darkness".to_string(), line.peek());
    }
}