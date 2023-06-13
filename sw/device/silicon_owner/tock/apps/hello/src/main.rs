//! An extremely simple libtock-rs example. Just prints out a message
//! using the Console capsule, then terminates.

#![no_main]
#![no_std]
use core::fmt::Write;
use libtock::console::Console;
use libtock::runtime::{set_main, stack_size};

pub mod console;
pub mod memory_cmd;

use crate::console::{ConsoleCommand, ConsoleError, DebugConsole};
use crate::memory_cmd::mem_cmd;

set_main!(main);
stack_size!(0x400);

const HELLO_PROMPT: &'static str = "\r
    | | | |  \r
  +---------+\r
--| OT      |--\r
--| Debug   |--\r
--| Console |--\r
--|         |--\r
  +---------+\r
    | | | |\r";

const HELLO_CMD_HELP: &'static str = "Write hello prompt";

pub fn hello(_args: &[&str]) -> Result<(), ConsoleError> {
    write!(Console::writer(), "{}\r\n", &HELLO_PROMPT).unwrap();
    Ok(())
}

const CONSOLE_BUFFER_SIZE: usize = 128;

fn main() {
    hello(&[]);
    let mut dc = DebugConsole::new(&[
        ConsoleCommand {
            name: "hello",
            exec: hello,
        },
        ConsoleCommand {
            name: "mem",
            exec: mem_cmd,
        },
    ]);
    let mut line_buffer: [u8; CONSOLE_BUFFER_SIZE] = [0; CONSOLE_BUFFER_SIZE];
    loop {
        dc.print_prompt();
        let ret = dc.read_line(&mut line_buffer);
        match ret {
            Ok(line) => {
                dc.execute_cmd(&line).unwrap();
            }
            Err(_err) => {
                write!(Console::writer(), "{}\r\n", &HELLO_PROMPT).unwrap();
            }
        }
    }
}
