use sysinfo::SystemExt;
use signal_hook::{iterator, consts::{SIGINT, SIGALRM, SIGQUIT}};
use std::{env, process, thread, error::Error, io::{self, Write}};
use nix::unistd::{alarm, Pid};
use nix::sys::signal::{self, Signal};
use users;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut timeout = 100u32;
    if args.len() == 2 {
        timeout = args[1].to_string()
                        .parse::<u32>()
                        .unwrap();
    }

    if let Err(_) = register_signal_handlers() {
        println!("Signals are not handled properly");
    }
    
    loop {
        alarm::cancel(); // cancel if other alarm process is running
        execute_shell(timeout);
    }
}

/// Register UNIX system signals
fn register_signal_handlers() -> Result<(), Box<dyn Error>>  {
    let mut signals = iterator::Signals::new(&[SIGINT, SIGALRM, SIGQUIT])?;

    // signal execution is passed to the child process
    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGALRM => {
                    write_to_stdout("This's taking too long...\n").unwrap(); // not safe
                    // when alarm goes off it kills child process
                    signal::kill(Pid::from_raw(0), Signal::SIGINT).unwrap()
                },
                SIGQUIT => {
                    write_to_stdout("Good bye!\n").unwrap(); // not safe
                    process::exit(0);
                },
                SIGINT => assert_ne!(0, sig), // assert that the signal is sent
                _ => continue,
            }
        }
    });

    Ok(())
}

/// Run the minishell
fn execute_shell(timeout: u32) {
    let minishell = build_user_minishell();
    match write_to_stdout(&minishell) {
        Ok(v) => v,
        Err(e) =>  {
            eprintln!("Unable to write to stdout : {}", e);
            process::exit(1);
        },
    }

    let cmd = get_user_command();
    alarm::set(timeout);
    if let Err(_) = process::Command::new(&cmd).status() {
        eprintln!("{}: command not found!", &cmd);
    }

}

/// flushes text buffer to the stdout
fn write_to_stdout(text: &str) -> io::Result<()> {
    io::stdout().write(text.as_ref())?;
    io::stdout().flush()?; // to the terminal
    Ok(())
}

/// fetch the user inputted command
fn get_user_command() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    while input.ends_with('\n') {
        input.pop();
    }
    remove_whitespace(&mut input);
    input
}

/// build a minishell name for the display
fn build_user_minishell() -> String {
    let mut username = String::new();

    // get user name
    let u = users::get_user_by_uid(
        users::get_current_uid()
    ).unwrap();

    username.push_str(&u.name().to_string_lossy());
    username.push_str("@");

    // get system name
    let system = sysinfo::System::new_all();
    username.push_str(&system.get_name().unwrap());

    username.push_str("# ");
    username
}

/// Function to remove leading and trailing white spaces from string
fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}