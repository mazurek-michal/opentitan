// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;

use crate::io::gpio::{GpioPin, PinMode, PullMode};
use std::cell::RefCell;
use std::rc::Rc;

use crate::transport::ti50::control::ControlInterface;
use crate::transport::ti50::transport::Ti50;

pub struct Ti50Gpio {
    ctl: Rc<RefCell<ControlInterface>>,
    pin_id: String
}

impl Ti50Gpio {
    pub fn open(ti: &Ti50, pin_id: &str) -> Result<Self>
    {
        Ok(Ti50Gpio {
            ctl: ti.control_interface.clone(),
            pin_id: String::from(pin_id),
        })
    }
}

impl GpioPin for Ti50Gpio {
    /// Reads the value of the the GPIO pin `id`.
    fn read(&self) -> Result<bool> {
        Ok(true)
    }

    /// Sets the value of the GPIO pin `id` to `value`.
    fn write(&self, value: bool) -> Result<()> {
        Ok(())
    }

    fn set_mode(&self, mode: PinMode) -> Result<()> {
        Ok(())
    }

    fn set_pull_mode(&self, mode: PullMode) -> Result<()> {
        Ok(())
    }
}


