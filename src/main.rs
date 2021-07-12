// use sysinfo::SystemExt;
pub mod tokenizer;

use crate::tokenizer::*;
use signal_hook::{
    consts::{SIGINT, SIGQUIT},
    iterator,
};
use std::fs::{File, OpenOptions};
use std::{
    error::Error,
    io::{self, Write},
    process, thread,
};


fn main() {
    if let Err(_) = register_signal_handlers() {
        println!("Signals are not handled properly");
    }

    loop {
        execute_shell();
    }
}

/// Register UNIX system signals
fn register_signal_handlers() -> Result<(), Box<dyn Error>> {
    let mut signals = iterator::Signals::new(&[SIGINT, SIGQUIT])?;

    // signal execution is passed to the child process
    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGQUIT => process::exit(0),
                SIGINT => assert_eq!(2, sig), // assert that the signal is sent
                _ => continue,
            }
        }
    });

    Ok(())
}

/// Run the minishell to execute user supplied instructions
fn execute_shell() {
    let minishell = "ghost# ";
    write_to_stdout(&minishell).expect("Unable to write to standard output");

    let mut cmd_line = get_user_commands();
    let mut redirection_count= [0; 2];

    if cmd_line.has_redirection() {
        let args = cmd_line.args_before_redirection();
        let mut proc = process::Command::new(&args[0]);

        proc.args(&args[1..]);

        loop {
            if cmd_line.peek().eq("<") {
                cmd_line.next(); // skip redirection character
                redirection_count[0] += 1;

                // retrieve file name if file/directory doesn't
                // exist notify user and restart the shell
                if let Some(name) = cmd_line.next() {
                    // redirect stdin from a given file
                    if let Ok(file_in) = open_stdin_file(&name) {
                        proc.stdin(file_in);
                    } else {
                        eprintln!("{}: No such file or directory", name);
                        return;
                    }
                };
            }

            if cmd_line.peek().eq(">") {
                cmd_line.next(); // skip redirection character
                redirection_count[1] += 1;

                // redirect stdout to a give file
                if let Some(name) = cmd_line.next() {
                    if let Ok(file_out) = open_stdout_file(&name) {
                        proc.stdout(file_out);
                    } else {
                        eprintln!("{}: No such file or directory", name);
                        return;
                    }
                }
            }

            if cmd_line.peek().is_empty() {
                break;
            }
        }

        // check flags that we don't have excessive number of redirections
        if redirection_count[0] > 1 || redirection_count[1] > 1 {
            eprintln!("invalid instructions for stream redirection");
            return;
        }
        // create child process and execute command
        // after that wait for the process to complete
        if let Ok(mut c) = proc.spawn() {
            c.wait().unwrap();
        } else {
            eprintln!("{}: command not found!", &args[0]);
        }
    } else {
        // execute command that has no redirections
        let cmd = cmd_line.get_args();
        if let Err(_) = process::Command::new(&cmd[0])
                                        .args(&cmd[1..])
                                        .status() {
            eprintln!("{}: command not found!", &cmd[0]);
        }
    }
}

/// Redirect a std out to a give file.
/// If file doesn't exists create one
fn open_stdout_file(file_name: &str) -> Result<File, io::Error> {
    let file = OpenOptions::new()
                            .truncate(true)
                            .write(true)
                            .create(true)
                            .open(file_name)?;
    Ok(file)
}

/// Redirect a std in from a given file to console.
/// If file doesn't exist error is thrown
fn open_stdin_file(file_name: &str) -> Result<File, io::Error> {
    let file = OpenOptions::new()
                            .read(true)
                            .open(file_name)?;
    Ok(file)
}

/// flushes text buffer to the stdout
fn write_to_stdout(text: &str) -> io::Result<()> {
    io::stdout().write(text.as_ref())?;
    io::stdout().flush()?; // to the terminal
    Ok(())
}

/// fetch the user inputted commands
fn get_user_commands() -> Tokenizer {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.ends_with('\n') {
        input.pop();
    }

    Tokenizer::new(&input)
}
