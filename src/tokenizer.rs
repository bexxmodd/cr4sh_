/// struct that reads a line of string splits on
/// white space and creates iterator of tokens
#[derive(Debug)]
pub struct Tokenizer {
    // holds string as an option and splits on 
    // whitespace only when next() is called.
    current: Option<String>,
}

impl Tokenizer {
    /// constractor
    pub fn new(line: &str) -> Self {
        Tokenizer {
            current: Some(line.to_string()),
        }
    }

    /// If the `current` line contains redirection ">" or "<"
    /// returns all tokens before redirection as a vector of strings.
    /// Else is found returns all the tokens as vector of strings. 
    /// This method call consumes tokens from `current`
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

    /// If the `current` line has a pipe symbol "|" in it
    /// this method will return part of string string before
    /// that symbol as a new Tokenizer object. This method consumes
    /// `current` line, so if no pipe symbol is found, it will
    /// reconstruct a new Tokenizer while disposing current one.
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

    /// get all the argument from the `current` line
    /// and return as a vector of strings.
    pub fn get_args(&mut self) -> Vec<String> {
        let mut args = vec![];
        while let Some(a) = self.next() {
            args.push(a);
        }
        args
    }

    /// checks if the `current` Tokenizer has pipe directive
    pub fn is_pipe(&self) -> bool {
        self.contains("|")
    }

    /// check if the `current` Tokenizer has redirection directive
    pub fn has_redirection(&self) -> bool {
        self.contains(">") || self.contains("<")
    }

    /// checks if the `current` contains given string pattern
    pub fn contains(&self, pattern: &str) -> bool {
        if let Some(cur) = self.current.as_ref() {
            cur.contains(pattern)
        } else {
            false
        }
    }

    /// peek what is the next token without consuming it.
    /// this returns a copy of the next token.
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
}