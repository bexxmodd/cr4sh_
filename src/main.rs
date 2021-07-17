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

    if cmd_line.is_pipe() {
        if let Err(e) = piped_cmd_execution(&mut cmd_line) {
            eprintln!("Command can't be executed!: {}", e);
        }
        return;
    } else if cmd_line.has_redirection() {
        let cmd = cmd_line.peek().clone();
        let mut proc = match redirect_cmd_execution(&mut cmd_line) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };
        if let Ok(mut c) = proc.spawn() {
            c.wait().unwrap();
        } else {
            eprintln!("{}: command not found!", cmd);
        }
    } else {
        // execute command that has no redirections
        let cmd = cmd_line.get_args();
        if let Err(_) = process::Command::new(&cmd[0]).args(&cmd[1..]).status() {
            eprintln!("{}: command not found!", &cmd[0]);
        }
    }
}

/// If user supplies piped command this function splits it into
/// two processes, executes them and pipes one being intput to the pipe
/// and the other being output from the pipe, which ends up displayed
pub fn piped_cmd_execution(cmd_line: &mut Tokenizer) -> Result<(), io::Error> {
    let mut tokens_before_pipe = cmd_line.commands_before_pipe();

    let mut after_pipe_cmd: Vec<String> = vec![];
    let mut before_pipe_cmd: Vec<String> = vec![];

    let mut proc = if cmd_line.has_redirection() {
        redirect_cmd_execution(cmd_line)?
    } else {
        after_pipe_cmd = cmd_line.get_args();
        // create a child process which will have input end of pipe open for stream
        process::Command::new(&after_pipe_cmd[0])
    };

    // check if we have any arguments othwerise execute command
    if after_pipe_cmd.len() > 0 {
        proc.args(&after_pipe_cmd[1..]);
    }
    let child = proc.stdin(process::Stdio::piped()).spawn();

    let mut proc2 = if tokens_before_pipe.has_redirection() {
        redirect_cmd_execution(&mut tokens_before_pipe)?
    } else {
        before_pipe_cmd = tokens_before_pipe.get_args();
        // create child process that redirects its output
        // to the stdout end of the pipe.
        // this will execute command and send output
        // to the previously created process pipe.
        process::Command::new(&before_pipe_cmd[0])
    };

    // check for arguments
    if before_pipe_cmd.len() > 1 {
        proc2.args(&before_pipe_cmd[1..]);
    }

    proc2.stdout(child.unwrap().stdin.unwrap()).output()?;
    Ok(())
}


/// If the user command has stream redirection this function is used
/// to accomodate that. This is done by creating a redirection and returing
/// a command which can then be spawned as a child processes
pub fn redirect_cmd_execution(cmd_line: &mut Tokenizer) -> Result<process::Command, io::Error> {
    let mut redirection_count = [0; 2];
    let args = cmd_line.args_before_redirection();

    // create process that will execute shell command
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
                match open_stdin_file(&name) {
                    Ok(f) => proc.stdin(f),
                    Err(e) => {
                        eprintln!("{}: No such file or directory", name);
                        return Err(e);
                    }
                };
            };
        }

        if cmd_line.peek().eq(">") {
            cmd_line.next(); // skip redirection character
            redirection_count[1] += 1;

            // redirect stdout to a give file
            if let Some(name) = cmd_line.next() {
                match open_stdout_file(&name) {
                    Ok(f) => proc.stdout(f),
                    Err(e) => {
                        eprintln!("{}: No such file or directory", name);
                        return Err(e);
                    }
                };
            }
        }

        if cmd_line.peek().is_empty() {
            break;
        }
    }

    // check flags that we don't have excessive number of redirections
    if redirection_count[0] > 1 || redirection_count[1] > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid instruction for redication",
        ));
    }

    Ok(proc)
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
    let file = OpenOptions::new().read(true).open(file_name)?;
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

    // read user input
    io::stdin().read_line(&mut input).unwrap();
    if input.ends_with('\n') {
        input.pop();
    }

    Tokenizer::new(&input)
}
