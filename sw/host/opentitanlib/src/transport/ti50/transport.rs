// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
//use log::info;
use std::cell::RefCell;
use std::rc::Rc;
//use std::time::Duration;

use crate::io::uart::Uart;
use crate::io::gpio::GpioPin;
use crate::io::emulator::Emulator;
use crate::transport::{Capabilities, Capability, Transport, TransportError};

use crate::transport::ti50::control::ControlInterface;
use crate::transport::ti50::gpio::Ti50Gpio;
use crate::transport::ti50::uart::Ti50Uart;
use crate::transport::ti50::emulator::Ti50Emulator;

#[derive(Default)]
struct Inner {
    gpio: Option<Rc<dyn GpioPin>>,
    uart: Option<Rc<dyn Uart>>,
    emu: Option<Rc<dyn Emulator>>,
}

/// Represents the Ti50 transport object
pub struct Ti50 {
    pub instance_id: String,
    pub control_interface: Rc<RefCell<ControlInterface>>,
    inner: RefCell<Inner>,
}

impl Ti50 {
    /// Creates a new `Ti50Simulator` struct from instance_id
    pub fn new(instance_id: Option<String>) -> Result<Self> {
        if let Some(id) = instance_id {
            Ok(Ti50 {
                instance_id: id.clone(),
                control_interface: Rc::new(RefCell::new(ControlInterface::connect(&id.clone())?)),
                inner: Default::default()
            })
        }
        else {
            Err(TransportError::InvalidInstance("instance_id", "None".to_string()).into())
        }
    }
}

impl Transport for Ti50 {
    fn capabilities(&self) -> Capabilities {
        Capabilities::new(Capability::UART | Capability::GPIO | Capability::EMU )
    }

    fn uart(&self, instance: &str) -> Result<Rc<dyn Uart>> {
        ensure!(
            instance == "0",
            TransportError::InvalidInstance("uart", instance.to_string())
        );
        let mut inner = self.inner.borrow_mut();
        if inner.uart.is_none() {
            inner.uart = Some(Rc::new(Ti50Uart::open(self)?));
        }
        Ok(Rc::clone(inner.uart.as_ref().unwrap()))
    }

    fn gpio_pin(&self, instance: &str) -> Result<Rc<dyn GpioPin>> {
        let mut inner = self.inner.borrow_mut();
        if inner.gpio.is_none() {
            inner.gpio = Some(Rc::new(Ti50Gpio::open(self, instance)?));
        }
        Ok(Rc::clone(inner.gpio.as_ref().unwrap()))
    }

    fn emulator(&self) -> Result<Rc<dyn Emulator>> {
        let mut inner = self.inner.borrow_mut();
        if inner.emu.is_none() {
            inner.emu = Some(Rc::new(Ti50Emulator::open(self)?));
        }
        Ok(Rc::clone(inner.emu.as_ref().unwrap()))
    }
}
