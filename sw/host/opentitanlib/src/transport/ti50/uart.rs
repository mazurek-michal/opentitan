// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::time::Duration;

use crate::io::uart::Uart;
use crate::util::file;

use crate::transport::ti50::control::ControlInterface;
use crate::transport::ti50::transport::Ti50;

/// Represents the Ti50Uart virtual UART.
pub struct Ti50Uart {
    ctl: Rc<RefCell<ControlInterface>>,
}

impl Ti50Uart {
    pub fn open(ti: &Ti50) -> Result<Self> {
        Ok(Ti50Uart {
            ctl: ti.control_interface.clone(),
        })
    }
}

impl Uart for Ti50Uart {
    fn get_baudrate(&self) -> u32 {
        // The verilator UART operates at 7200 baud.
        // See `sw/device/lib/arch/device_sim_verilator.c`.
        7200
    }

    fn set_baudrate(&self, _baudrate: u32) -> Result<()> {
        // As a virtual uart, setting the baudrate is a no-op.
        Ok(())
    }

    fn read_timeout(&self, buf: &mut [u8], timeout: Duration) -> Result<usize> {
        //let mut file = self.file.borrow_mut();
        //file::wait_read_timeout(&*file, timeout)?;
        //Ok(file.read(buf)?)
        Ok(0)
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        //Ok(self.file.borrow_mut().read(buf)?)
        Ok(0)
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
        //self.file.borrow_mut().write_all(buf)?;
        //Ok(buf.len())
        Ok(0)
    }
}
