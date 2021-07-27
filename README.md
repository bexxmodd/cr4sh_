# cr4sh_ 0.0.1

**cr4sh_** (pronounced _crash_, because it crashes all the time) is a UNIX mini-shell implemented with Rust.

Currently, `cr4sh_` provides the following functionality:

- Multiprocessing
- SIGINT and SIGKILL signal handling
- Execution of OS executables
- Redirection of standard input & output
- Appending stdout to the file
- Piping commands and combining with redirection
- `cd` command to change directories
- `touch` for creating files and updating accessed & modified dates
- `&&` calls to chain multiple commands

<br>

### Demo: 

[![asciicast](https://asciinema.org/a/aDG5n2136psEN4rnt6oqb9i6v.svg)](https://asciinema.org/a/aDG5n2136psEN4rnt6oqb9i6v)

<br>

## Installation

To run the `cr4sh_` shell copy following code in your terminal:

```bash
git clone https://github.com/bexxmodd/cr4sh_.git && \
cd cr4sh_ && \
cargo run
```

<br>

## Plans

<br>
Currently I'm looking to add other custom shell functions (which are listed in the TODO list) and if you want to contribute looking into the `customs` directory inside the `src` directory is a great start. If you want just add any of the functionality and I'll take of gluing with the rest of the program. Also, if you have design skills some 80's style logo can be a great contribution!
<br>
<br>

Main priority right now is to add cursor and history function.
<br>

### TODO:

- [ ] Create a logo
- [x] Allow piping of the commands
- [x] Add personalized color printing of the terminal user
- [x] Allow execution of local executables properly
- [x] Allow chain of commands when `&&` is supplied
- [x] Handle append (`>>`) directive
- [ ] Handle `&` symbol to send command as a background process
- [ ] Expend signal handling capabilities
- [ ] Implement cursor to handle arrow, home, end keyboard inputs and cursor movement
- [ ] Usage of Tab to autocomplete commands and file/directory names
- [x] Implement `touch` function:
- [ ] Implement `history` function
- [ ] Implement `dot/source` function
- [ ] Implement redirection for custom functions:
    - [ ] add additional argument to functions for stdout file
- [ ] Implement piping for custom functions
- [ ] Add customization of colors and style for a shell-name
