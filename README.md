dbgcmd
===

[![Build Status](https://travis-ci.org/mistodon/dbgcmd.svg?branch=master)](https://travis-ci.org/mistodon/dbgcmd)
[![Crates.io](https://img.shields.io/crates/v/dbgcmd.svg)](https://crates.io/crates/dbgcmd)
[![Docs.rs](https://docs.rs/dbgcmd/badge.svg)](https://docs.rs/dbgcmd/0.1.0/dbgcmd/)

This is a simple library for implementing command-line-style debug consoles within an application.

It doesn't handle rendering, or the logic of any individual commands. All it does is model the
state of the console, including command history, and whether or not it is active. You define
your own command struct/enum, and drive the inputs to the console. After that, you can use the
`confirm` method to parse the text entered so far into a command.

When running in debug mode (when the `debug_assertions` flag is set) the entire Console is
disabled. This is to avoid shipping something and accidentally including the debug console.

To override this, you can use the `force-enabled` feature, which will allow the console to
work in release mode.

# Examples

## Input

```rust
use std::str::FromStr;
use dbgcmd::Console;

#[derive(Debug, PartialEq, Eq)]
enum Command {
    SayHi,
    SayBye,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hi" => Ok(Command::SayHi),
            "bye" => Ok(Command::SayBye),
            _ => Err(())
        }
    }
}

let mut console = Console::new();

// Your application passes along key inputs
console.receive_char('h');
console.receive_char('i');

match console.confirm().unwrap() {
    Command::SayHi => println!("Hi!"),
    Command::SayBye => println!("Bye!"),
}
```

## History

```rust
use dbgcmd::Console;

type Command = String;

let mut console = Console::new();

// Set the entire entry instead of individual characters
console.set_entry("first".to_owned());
console.confirm::<Command>();

console.set_entry("second".to_owned());
console.confirm::<Command>();

console.set_entry("third".to_owned());
console.confirm::<Command>();

// Scroll up or down the history
console.up();
assert_eq!(console.entry(), "third".to_owned());

console.up();
assert_eq!(console.entry(), "second".to_owned());

console.up();
assert_eq!(console.entry(), "first".to_owned());

console.down();
console.down();
console.down();
assert!(console.entry().is_empty());
```
