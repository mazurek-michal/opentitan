use core::fmt::Write;
use libtock::console::Console;
use crate::console::{parse_number, ConsoleError};

const MEM_CMD_HELP: &'static str = "Memory read/write command\r
mem read {address} {length}\r
mem write {address} {value1}...\r";

pub fn mem_cmd(args: &[&str]) -> Result<(), ConsoleError> {
    match args {
        [_, "read", address, length] => {
            match (parse_number(address), parse_number(length)) {
                (Some(addr), Some(len)) => {
                    // align adress to u32
                    let aligned_address = addr & !0xf;
                    write!(Console::writer(), "read address: 0x{:x} len:{}\n\r", aligned_address, len).unwrap();
                    let mut count = 0;
                    while count < len {
                        let p = aligned_address + count;
                        // SAFETY This is test/debug function. Intentional dereference of calculated address.
                        // TockOS should handled invalid read as panic and print debug info after killing
                        // applications
                        let data = unsafe { core::ptr::read_volatile(p as *const u8) };
                        match count % 4 {
                            0 => {
                                write!(Console::writer(), "0x{:x}: {:02x} ", p, data).unwrap();
                            },
                            3 => {
                                write!(Console::writer(), "{:02x}\n\r", data).unwrap();
                            },
                            _ => {
                                write!(Console::writer(), "{:02x} ", data).unwrap();
                            },
                        }
                        count += 1;
                    }
                    return Ok(())
                }
                _ => {
                    return Ok(())
                }
            }
        },
        [_, "write", address, ..] => {
            match parse_number(address) {
                Some(addr) => {
                    let aligned_address = addr & !0xf;
                    let number_of_values = args.len() - 3;
                    write!(Console::writer(), "write address: 0x{:x} length:{}\n\r", addr, number_of_values).unwrap();
                    for count in 0..number_of_values {
                        let p = aligned_address + count;
                        if let Some(value) = parse_number(args[3+count]) {
                            if value <= 0xff {
                                // SAFETY This is test/debug function. Intentional dereference of calculated address.
                                // TockOS should handled invalid write as panic and print debug info after killing
                                // applications
                                unsafe { core::ptr::write_volatile(p as *mut u8, value as u8) };
                            } else {
                                return Err(ConsoleError::InvalidValue)
                            }
                        } else {
                            return Err(ConsoleError::InvalidValue)
                        }
                    }
                    return Ok(())
                }
                _ => {
                    return Ok(())
                }
            }
        },
        [_, ..] => {
            return Err(ConsoleError::InvalidSubcomand)
        },
        _ => {
            return Err(ConsoleError::InvalidArgNumber)
        },
    }
}

