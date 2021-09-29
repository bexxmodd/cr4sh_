
/// struct that reads a line of string splits on
/// white space and creates iterator of tokens
#[derive(Clone, Debug)]
pub struct Tokenizer {
    // holds string as an option and splits on
    // whitespace only when next() is called.
    current: Option<String>,
    is_pipe: bool,
    has_redirection: bool,
}

impl Tokenizer {
    /// constructor
    pub fn new(line: &str) -> Self {
        let is_pipe = line.contains(" | ");
        let has_redirection = line.contains(" >") || line.contains(" < ");

        Tokenizer {
            current: Some(line.to_string()),
            is_pipe,
            has_redirection,
        }
    }

    /// If the `current` line contains redirection ">" or "<"
    /// returns all tokens before redirection as a vector of strings.
    /// Else is found returns all the tokens as vector of strings.
    /// This method call consumes tokens from `current`
    pub fn args_before_redirection(&mut self) -> Vec<String> {
        if !self.has_redirection() {
            return self.get_args()
        }

        let mut args = vec![];
        while self.current.is_some() {
            if self.peek().eq(">") ||
            self.peek().eq("<") ||
            self.peek().eq(">>") {
                break;
            } else {
                args.push(self.next().unwrap());
            }
        }
        args
    }

    /// If the `current` line has a pipe symbol "|" in it
    /// this method will return part of string string before
    /// that symbol as a new Tokenizer object. This method consumes
    /// `current` line, so if no pipe symbol is found, it will
    /// reconstruct a new Tokenizer while disposing current one.
    pub fn commands_before_pipe(&mut self) -> Tokenizer {
        if !self.is_pipe() {
            self.clone()
        } else {
            self._split_tokenizer("|")
        }
    }

    /// get all the argument from the `current` line
    /// and return as a vector of strings.
    pub fn get_args(&mut self) -> Vec<String> {
        let mut args = vec![];
        while let Some(a) = self.next() {
            if a.eq("&&") {
                break;
            }
            args.push(a);
        }
        args
    }

    /// checks if the `current` Tokenizer has pipe directive
    pub fn is_pipe(&self) -> bool {
        self.is_pipe
    }

    /// check if the `current` Tokenizer has redirection directive
    pub fn has_redirection(&self) -> bool {
        self.has_redirection
    }

    /// checks if the `current` contains given string pattern
    pub fn contains(&self, pattern: &str) -> bool {
        if let Some(cur) = self.current.as_ref() {
            cur.contains(pattern)
        } else {
            false
        }
    }

    /// Check if the current lines starts with a given prefix
    pub fn starts_with(&self, prefix: &str) -> bool {
        if let Some(cur) = self.current.as_ref() {
            if prefix.len() > cur.len() {
                return false
            }
            if prefix.eq(&cur[..prefix.len()]) {
                return true
            }
        }
        false
    }

    /// peek what is the next token without consuming it.
    /// this returns a copy of the next token.
    pub fn peek(&self) -> String {
        let mut res = String::new();
        if let Some(cur) = self.current.as_deref() {
            let mut open = 0u8;
            for c in cur.chars().into_iter() {
                if c.eq(&'"') || c.eq(&'\'') {
                open = open ^ 1;
                } else if c.eq(&' ') && open == 0 {
                    break;
                } else {
                    res.push(c);
                }
            }
        }
        res
    }

    pub fn get_multiple_tokens(&mut self, pattern: &str) -> Vec<Tokenizer> {
        let mut toks: Vec<_> = vec![];
        while self.current.is_some() {
            toks.push(self._split_tokenizer(pattern))
        }
        toks
    }

    pub fn _split_tokenizer(&mut self, pattern: &str) -> Tokenizer {
        let mut before = String::new();
        while let Some(a) = self.next() {
            if a.eq(pattern) {
                break;
            }
            // if next value has space in it, that means user supplied
            // text in quotation marks, & we preserve it in a new Tokenizer
            if a.contains(' ') {
                before.push('\'');
                before.push_str(&a);
                before.push('\'');
            } else {
                before.push_str(&a);
            }
            before.push(' ');
        }
        before.pop();
        Tokenizer::new(&before)
    }

    pub fn is_empty(&self) -> bool {
        self.current.is_none()
    }
}

impl Iterator for Tokenizer {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut stop = usize::MAX;
        let mut nxt= String::new();
        let mut remainder = String::new();

        if let Some(s) = &mut self.current {
            let mut open = 0u8;
            for (i, c) in s.chars().into_iter().enumerate() {
                if c.eq(&'"') || c.eq(&'\'') {
                    open = open ^ 1;
                } else if c.eq(&' ') && open == 0 {
                    stop = i + 1;
                    break
                } else {
                    nxt.push(c);
                }
            }
            if stop < s.len() {
                remainder = s[stop..].to_string();
            }
        }

        if remainder.is_empty() {
            self.current = None
        } else {
            self.current = Some(remainder);
        }
        if nxt.is_empty() {
            None
        } else { 
            Some(nxt)
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
    fn test_two_word_string() {
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
        let mut line = Tokenizer::new(&"Hello Darkness > \"My Oldie\"");
        assert_eq!("Hello".to_string(), line.peek());
        assert_eq!("Hello".to_string(), line.next().unwrap());
        assert_eq!("Darkness".to_string(), line.peek());
        line.next();
        line.next();
        assert_eq!("My Oldie".to_string(), line.peek());
    }

    #[test]
    fn test_contains() {
        let line = Tokenizer::new("This line tests $ symbol");
        assert!(line.contains(&"$"));
    }

    #[test]
    fn test_args_before_redirection() {
        let mut line = Tokenizer::new("Hello World > Bye");
        let v = line.args_before_redirection();
        assert_eq!(2, v.len());
        assert_eq!("Hello".to_string(), *v[0]);
        assert_eq!("World".to_string(), *v[1]);

        let a = line.get_args();
        assert_eq!(2, a.len());
        assert_eq!("Bye".to_string(), *a[1]);
    }

    #[test]
    fn test_commands_before_pipe() {
        let mut line = Tokenizer::new("ls -a | cat");
        let mut v = line.commands_before_pipe();
        assert_eq!("ls".to_string(), v.next().unwrap());
        assert_eq!("-a".to_string(), v.next().unwrap());
    }

    #[test]
    fn test_prefix() {
        let line = Tokenizer::new("this line starts with this");
        assert!(line.starts_with("this"));
    }
    
    #[test]
    fn test_quotation_marks() {
        let mut line = Tokenizer::new("echo \"Hello World\" 'Rust Lang' Yay!");
        
        assert_eq!("echo".to_string(), line.next().unwrap());
        assert_eq!("Hello World".to_string(), line.next().unwrap());
        assert_eq!("Rust Lang".to_string(), line.next().unwrap());
        assert_eq!("Yay!".to_string(), line.next().unwrap());
        assert_eq!(None, line.next());
    }
}
