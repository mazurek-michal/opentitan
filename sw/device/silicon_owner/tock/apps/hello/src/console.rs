use core::fmt::Write;
use core::str::FromStr;
use libtock::alarm::{Alarm, Milliseconds};
use libtock::console::Console;
use libtock::platform::ErrorCode;
use libtock::runtime::{set_main, stack_size};

const DEBUG_CONSOLE_BUFFER_SIZE: usize = 128;

pub struct ConsoleCommand<'a> {
    pub name: &'a str,
    pub exec: fn(args: &[&str]) -> Result<(), ConsoleError>,
}

pub enum ConsoleError {
    InvalidArgNumber,
    InvalidValue,
    InvalidSubcomand,
    Sys(ErrorCode)
}

pub fn parse_number<'a>(token: &str) -> Option<usize> {
    let mut radix: u32 = 10;
    let mut num = token;
    if token.starts_with("0x") {
        radix = 16;
        num = &token[2..];
    }
    return match usize::from_str_radix(&num, radix) {
        Ok(val) => Some(val),
        _ => None,
    };
}

pub fn is_whitespace(byte: u8) -> bool {
    match byte {
        b' ' | b'\n' | b'\t' => true,
        _ => false,
    }
}

pub struct DebugConsole<'cmd> {
    command_table: &'cmd [ConsoleCommand<'cmd>],
}

impl<'cmd> DebugConsole<'cmd> {
    const PROMPT_TEXT: &'static str = "\r\nOTDC:>";
    const CR: u8 = b'\r'; // carige return
    const LF: u8 = b'\n'; // line feed
    const BS: u8 = b'\x08'; // backspace
    const DEL: u8 = b'\x7f'; // delete
    const ESC: u8 = b'\x1B'; // ESC - start escape sequence

    pub fn new(commands: &'cmd [ConsoleCommand<'cmd>]) -> DebugConsole<'cmd> {
        Self {
            command_table: commands,
        }
    }

    pub fn print_prompt(&self) {
        Console::write(&DebugConsole::PROMPT_TEXT.as_bytes()).unwrap();
    }

    pub fn cleare_line(&self) {
        write!(Console::writer(), "\x1b[2K").unwrap();
    }

    pub fn read(&self) -> Result<u8, ErrorCode> {
        let mut buffer: [u8; 4] = [0; 4];
        // ugly hack - TockOs wait until buffer is full so we need only onie element buffer
        // to not hung in syscall forever. Also TockOS console don't provide echo.
        let (len, ret) = Console::read(&mut buffer[0..1]);
        match (len, ret) {
            (1, Err(ErrorCode::Size)) => Ok(buffer[0]),
            (_, Err(err)) => Err(err),
            // TODO check if code should panic heare
            (_, Ok(_)) => Ok(buffer[0]),
        }
    }

    pub fn read_line<'a>(&mut self, buffer: &'a mut [u8]) -> Result<&'a [u8], ErrorCode> {
        let mut count: usize = 0;
        // clean buffer
        for byte in buffer.iter_mut() {
            *byte = 0;
        }
        // read as manu char as posible
        while count < buffer.len() {
            match self.read() {
                Err(err) => {
                    return Err(err);
                }
                Ok(DebugConsole::CR) => {
                    if count > 0 {
                        Console::write(b"\n\r");
                    }
                    return Ok(&buffer[0..count]);
                }
                Ok(DebugConsole::LF) => {
                    // Drop
                }
                Ok(DebugConsole::ESC) => {
                    write!(Console::writer(), "ESC").unwrap();
                }
                Ok(DebugConsole::BS) | Ok(DebugConsole::DEL) => {
                    if count > 0 {
                        count -= 1;
                        buffer[count] = b' ';
                        // TODO clear display
                        write!(Console::writer(), "\x1b[2K").unwrap();
                        Console::write(b"\rOTDC:>");
                        Console::write(&buffer[0..count]).unwrap();
                        //move cursor 1 left
                        //Console::write(b"\x1bD").unwrap();
                    }
                }
                Ok(c) if c >= 0x20 && c <= 0x7e => {
                    // echo
                    buffer[count] = c;
                    count += 1;
                    Console::write(&buffer[(count - 1)..count]).unwrap();
                }
                Ok(x) => {
                    write!(Console::writer(), "0x{:02x}", x as usize).unwrap();
                }
            }
        }
        return Ok(&buffer[0..count]);
    }

    // parse line in place to tokens
    pub fn parse_cmd<'a>(
        line: &'a [u8],
        token_buffer: &'a mut [&'a str],
    ) -> &'a [&'a str] {
        let mut index: usize = 0;
        let mut token_counter: usize = 0;
        let mut old: u8 = b' ';
        let mut new: u8 = b' ';
        let mut start: usize = 0;
        let mut len: usize = 0;
        while index < line.len() && token_counter < token_buffer.len() {
            old = new;
            new = line[index];
            //TODO change for match expression ?
            if is_whitespace(old) && !is_whitespace(new) {
                // token start
                start = index;
                len = 1;
            }
            if !is_whitespace(old) && !is_whitespace(new) {
                // token body
                len += 1;
            }
            if !is_whitespace(old) && is_whitespace(new) {
                // token end
                unsafe {
                    token_buffer[token_counter] = core::str::from_utf8_unchecked(&line[start..start+len]);
                }
                token_counter += 1;
                // prepare new toke to be filled
                start = 0;
                len = 0;
            }
            index += 1;
        }
        if len > 0 {
            unsafe { token_buffer[token_counter] = core::str::from_utf8_unchecked(&line[start..start+len]); };
            token_counter += 1;
        }
        return &token_buffer[0..token_counter];
    }

    pub fn execute_cmd(&self, line: &[u8]) -> Result<(), ErrorCode> {
        if line.len() > 0 {
            let mut token_buffer: [&str; 16] = [""; 16];
            let args = Self::parse_cmd(&line, &mut token_buffer);
            if args.len() > 0 {
                write!(Console::writer(), "\n\rcmd:{:?}\r\n", &args).unwrap();
                for command in self.command_table {
                    if args[0] == command.name {
                        return match ((command.exec)(&args)) {
                            Err(ConsoleError::InvalidArgNumber) => {
                                write!(Console::writer(), "Invalid Number of args\n").unwrap();
                                Ok(())
                            },
                            Err(ConsoleError::InvalidValue) => {
                                write!(Console::writer(), "Invalid value\n").unwrap();
                                Ok(())
                            },
                            Err(ConsoleError::InvalidSubcomand) => {
                                write!(Console::writer(), "Invalid subcomand\n").unwrap();
                                Ok(())
                            },
                            Err(ConsoleError::Sys(err)) => {
                                write!(Console::writer(), "Syscal error: {}\n", err as usize).unwrap();
                                Err(err)
                            },
                            Ok(_) => Ok(()),
                        }
                    }
                }
                write!(Console::writer(), "Unknown command!!!:{}\n", args[0]).unwrap();
            }
        }
        Ok(())
    }
}
