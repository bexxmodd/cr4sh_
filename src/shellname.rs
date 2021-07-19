use sysinfo::SystemExt;

pub struct ShellName {
    name: String,
    current_dir: String,
    pub shell_name: String,
}

impl ShellName {
    pub fn new(current_dir: &str) -> Self {
        let user = build_user_minishell();
        ShellName {
            name: user.clone(),
            current_dir: current_dir.to_string(),
            shell_name: user + ":" + current_dir + "$ ",
        }
    }

    pub fn set_current_dir(&mut self, dir: &str) {
        let home = dirs::home_dir().unwrap();
        if let Some(h) = home.to_str() {
            if dir.starts_with(h) {
                self.current_dir = dir.replace(h, "~");
            } else {
                self.current_dir = dir.to_string();
            }
        }

        self.shell_name = self.name.to_string() + ":" + &self.current_dir + "$ ";
    }
}

/// build a minishell name for the display
fn build_user_minishell() -> String {
    let mut username = String::new();

    // get user name
    let u = users::get_user_by_uid(users::get_current_uid()).unwrap();

    username.push_str(&u.name().to_string_lossy());
    username.push_str("@");

    // get system name
    let system = sysinfo::System::new_all();
    username.push_str(&system.get_name().unwrap());

    username
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_shellname() {
        let sh = ShellName::new("home");
        assert_eq!("home".to_string(), sh.current_dir);
    }
}
