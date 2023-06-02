//! An extremely simple libtock-rs example. Just prints out a message
//! using the Console capsule, then terminates.

#![no_main]
#![no_std]
use core::fmt::Write;
use libtock::alarm::{Alarm, Milliseconds};
use libtock::console::Console;
use libtock::platform::ErrorCode;
use libtock::runtime::{set_main, stack_size};

pub mod console;
use crate::console::{DebugConsole, ConsoleCommand, CommandToken};

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

pub fn hello(line: &[u8], args: &[CommandToken]) -> Result<(), ErrorCode>{
    write!(Console::writer(), "{}\r\n", &HELLO_PROMPT).unwrap();
    Ok(())
}

const CONSOLE_BUFFER_SIZE: usize = 128;



fn main() {
    hello(b"", &[]);
    let mut dc = DebugConsole::new(&[ConsoleCommand {name: "hello", exec: hello} ]);
    let mut line_buffer: [u8;CONSOLE_BUFFER_SIZE] = [0;CONSOLE_BUFFER_SIZE];
    loop {
        dc.print_prompt();
        let ret = dc.read_line(&mut line_buffer);
        match ret {
            Ok(line) => {
                dc.execute_cmd(&line).unwrap();
            }
            Err(_err) => {
                write!(Console::writer(), "{}\r\n", &HELLO_PROMPT).unwrap();
                //TODO write error to low lewel debug or panic
            }
        }
    }
}
