use signal_hook::{iterator::Signals, consts::SIGINT};
use std::{process, thread, error::Error};
use std::io::{self, Write, Read};
use ctrlc;
use std::process::{Command};

fn main() {
    let handlers = register_signal_handlers();
    match handlers {
        Ok(h) => h,
        Err(_) => panic!("Signals not handled properly"),
    }
    let timeout = 0u32;

    loop { execute_shell(timeout); }
}

fn register_signal_handlers() -> Result<(), Box<dyn Error>> {
    let mut signals = Signals::new(&[SIGINT])?;

    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGINT => process::exit(SIGINT),
                _ => println!("Can't handle that yet"),
            }
        }
    });

    Ok(())
}

fn execute_shell(timeout: u32) {
    let minishell = "shredder# ";
    match write_to_stdout(minishell) {
        Ok(v) => v,
        Err(e) => println!("Unable to write to stdout : {}", e),
    }
    let cmd = get_command_from_input();
    if let Err(user_process) = Command::new(&cmd).status() {
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
