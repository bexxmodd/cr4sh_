# cr4sh_ 0.0.1

cr4sh_ (pronounced crash, because it crashes all the time) is a UNIX mini-shell implemented with Rust.

Currently, `cr4sh_` provides the following functionality:

- Multiprocessing
- Execution of OS executables
- Redirection of standard input output
- Appending stdout to the file
- Piping commands and combining with redirection
- `cd` command to change directories
- `touch` for creating files and updating accessed & modified dates
- `&&` calls to chain multiple commands

<br>

### Demo: 

[![asciicast](https://asciinema.org/a/aDG5n2136psEN4rnt6oqb9i6v.svg)](https://asciinema.org/a/aDG5n2136psEN4rnt6oqb9i6v)

<br>

To run the shell do the following steps:

1. copy current repo with `git clone
2. `cd` into `cr4sh_` directory
3. start the shell by running `cargo run`

Currently looking to add other custom functions (which are listed in TODO list) and if you want to contribute looking into the `customs` folder inside the `src` folder is a great start. If you want just add any of the functionality and I'll take of gluing with the rest of the program.

### TODO:

- [x] Allow piping of the commands
- [x] Add personalized color printing of the terminal user
- [x] Allow execution of local executables properly
- [x] Allow chain of commands when `&&` is supplied
- [x] Handle append (`>>`) directive
- [ ] Handle `&` symbol to send command as a background process
- [ ] Implement cursor to handle arrow, home, end keyboard inputs and cursor movement
- [x] Implement `touch` function:
- [ ] Implement `history` function
- [ ] Implement `dot/source` function
- [ ] Implement redirection for custom functions:
    - [ ] add additional argument to functions for stdout file
- [ ] Implement piping for custom functions
