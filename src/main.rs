use signal_hook::{iterator::Signals, consts::SIGINT};
use std::{process::Command, thread, error::Error, io::{self, Write}};
use users::{get_user_by_uid, get_current_uid};
use sysinfo::{SystemExt};

fn main() {
    if let Err(_) = register_signal_handlers() {
        println!("Signals are not handled properly");
    };
    let timeout = 0u32;

    loop { execute_shell(timeout); }
}

fn register_signal_handlers() -> Result<(), Box<dyn Error>>  {
    let mut signals = Signals::new(&[SIGINT])?;

    thread::spawn(move || {
        for sig in signals.forever() {
            assert_ne!(0, sig);
        }
    });

    Ok(())
}

fn execute_shell(timeout: u32) {
    let minishell = build_user_minishell();
    match write_to_stdout(&minishell) {
        Ok(v) => v,
        Err(e) => println!("Unable to write to stdout : {}", e),
    }

    let cmd = get_command_from_input();
    if let Err(_) = Command::new(&cmd).status() {
        println!("{}: command not found!", &cmd);
    }

}

fn write_to_stdout(text: &str) -> io::Result<()> {
    io::stdout().write(text.as_ref())?;
    io::stdout().flush()?;
    Ok(())
}

fn get_command_from_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.ends_with('\n') {
        input.pop();
    }
    input
}

fn build_user_minishell() -> String {
    let mut username = String::new();

    // get user name
    let u = get_user_by_uid(get_current_uid()).unwrap();
    username.push_str(&u.name().to_string_lossy());

    username.push_str("@");

    // get system name
    let system = sysinfo::System::new_all();
    username.push_str(&system.get_name().unwrap());

    username.push_str("# ");
    username
}
