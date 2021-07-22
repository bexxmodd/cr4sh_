pub mod cd;
pub mod shellname;
pub mod tokenizer;
pub mod touch;

#[macro_use]
extern crate lazy_static;

use crate::{shellname::*, tokenizer::*};
use signal_hook::{
    consts::{SIGINT, SIGQUIT},
    iterator,
};
use std::collections::HashSet;
use std::{
    env::current_dir,
    fs::{File, OpenOptions},
};
use std::{
    error::Error,
    io::{self, Write},
    process, thread,
};

lazy_static! {
    static ref CUSTOM_FN: HashSet<&'static str> = {
        vec!["cd", "source", "touch", "history"]
            .into_iter()
            .collect()
    };
}

fn main() {
    if let Err(_) = register_signal_handlers() {
        println!("Signals are not handled properly");
    }

    let cur = current_dir().unwrap();
    let cur = cur.strip_prefix(dirs::home_dir().unwrap()).unwrap();

    // create initial shell terminal display
    let mut minishell = ShellName::new(cur.to_str().unwrap());

    loop {
        execute_shell(&mut minishell);
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
fn execute_shell(shell_name: &mut ShellName) {
    write_to_stdout(&shell_name.shell_name).expect("Unable to write to stdout");

    let mut cmd_line = match get_user_commands() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    let tokens = cmd_line.get_multiple_tokens("&&");

    for mut token in tokens {
        if CUSTOM_FN.contains(&token.peek()[0..]) {
            if let Err(e) = execute_custom_fn(shell_name, &mut token) {
                eprint!("{}", e);
            }
            continue;
        } else if token.is_pipe() {
            if let Err(e) = piped_cmd_execution(&mut token) {
                eprintln!("Error: {}", e);
            }
            continue;
        } else if token.has_redirection() {
            let cmd = token.peek().clone();
            let mut proc = match redirect_cmd_execution(&mut token) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            };

            if let Ok(mut c) = proc.spawn() {
                c.wait().unwrap();
            } else {
                eprintln!("{}: command not found!", cmd);
            }
        } else {
            // execute command that has no redirection
            let cmd = token.get_args();
            if let Err(_) = process::Command::new(&cmd[0]).args(&cmd[1..]).status() {
                eprintln!("{}: command not found!", &cmd[0]);
            }
        }
    }
}

fn execute_custom_fn(shell_name: &mut ShellName,
    token: &mut Tokenizer) -> Result<(), io::Error> {
    match &token.peek()[0..] {
        "cd" => cd::change_directory(shell_name, token),
        "touch" => touch::touch(token)?,
        _ => println!("Not implemented yet"),
    }
    Ok(())
}

/// If user supplies piped command this function splits it into
/// two processes, executes them and pipes one being input to the pipe
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

    // check if we have any arguments otherwise execute command
    if after_pipe_cmd.len() > 0 {
        proc.args(&after_pipe_cmd[1..]);
    }
    let child = proc.stdin(process::Stdio::piped()).spawn()?;

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

    proc2.stdout(child.stdin.unwrap()).output()?;
    Ok(())
}

/// If the user command has stream redirection this function is used
/// to accommodate that. This is done by creating a redirection and returning
/// a command which can then be spawned as a child processes
pub fn redirect_cmd_execution(cmd_line: &mut Tokenizer) -> Result<process::Command, io::Error> {
    let mut redirection_count = [0; 2];
    let args = cmd_line.args_before_redirection();

    // create process that will execute shell command
    let mut proc = process::Command::new(&args[0]);

    proc.args(&args[1..]);

    loop {
        if cmd_line.peek().eq("<") {
            assert_eq!("<".to_string(), cmd_line.next().unwrap());
            redirection_count[0] += 1;

            // retrieve file name if file/directory doesn't
            // exist notify user and restart the shell
            if let Some(name) = cmd_line.next() {
                // redirect stdin from a given file
                match open_stdin_file(&name) {
                    Ok(f) => proc.stdin(f),
                    Err(e) => return Err(e),
                };
            };
        }

        if cmd_line.peek().eq(">") {
            assert_eq!(">".to_string(), cmd_line.next().unwrap());
            redirection_count[1] += 1;

            // redirect stdout to a give file
            if let Some(name) = cmd_line.next() {
                match open_stdout_file(&name) {
                    Ok(f) => proc.stdout(f),
                    Err(e) => return Err(e),
                };
            }
        }

        if cmd_line.peek().is_empty() {
            break;
        }
    }

    // check flags that we don't have excessive number of redirection
    if redirection_count[0] > 1 || redirection_count[1] > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid instructions for redirection",
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
fn get_user_commands() -> Result<Tokenizer, io::Error> {
    let mut input = String::new();

    // read user input
    io::stdin().read_line(&mut input).unwrap();

    if input.trim().len() < 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid command",
        ));
    }
    if input.ends_with('\n') {
        input.pop();
    }

    Ok(Tokenizer::new(input.trim()))
}
