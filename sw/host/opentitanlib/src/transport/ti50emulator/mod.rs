// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use log::{error, info};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::io::emu::Emulator;
use crate::io::gpio::GpioPin;
use crate::io::i2c::Bus;
use crate::io::spi::Target;
use crate::io::uart::Uart;
use crate::transport::{Capabilities, Capability, Result, Transport};

mod emu;
mod gpio;
mod i2c;
mod spi;
mod uart;

use crate::transport::ti50emulator::emu::Ti50Emu;

/// Implementation of the Transport trait backed by connection to a remote OpenTitan tool
/// session process.
pub struct Ti50Emulator {
    inner: Rc<RefCell<Inner>>,
}

impl Ti50Emulator {
    /// Establish connection with a running session process.
    pub fn open(
        executable_directory: PathBuf,
        executable: &String,
        instance_prefix: &String,
    ) -> anyhow::Result<Self> {
        let tstamp = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let instance_name = format!(
            "{}_{}_{}_{}",
            instance_prefix,
            process::id(),
            tstamp.as_secs(),
            tstamp.as_nanos()
        );

        let mut instance_directory = PathBuf::from("/tmp");
        instance_directory.push(&instance_name);
        let resource_directory = instance_directory.join("resources");
        let runtime_directory = instance_directory.join("runtime");

        info!("Initializing Ti50Emulator instance:{}", instance_name);
        fs::create_dir(&instance_directory)?;
        fs::create_dir(&resource_directory)?;
        fs::create_dir(&runtime_directory)?;

        Ok(Self {
            inner: Rc::new(RefCell::new(Inner {
                instance_directory: instance_directory.clone(),
                resource_directory: resource_directory.clone(),
                runtime_directory: runtime_directory.clone(),
                executable_directory: executable_directory,
                executable: Some(executable.clone()),
                emulator: None,
                spi_map: HashMap::new(),
                gpio_map: HashMap::new(),
                i2c_map: HashMap::new(),
                uart_map: HashMap::new(),
            })),
        })
    }
}

impl Drop for Ti50Emulator {
    fn drop(&mut self) {
        info!(
            "Clenup Ti50Emulator instance directory:{}",
            self.inner.borrow().instance_directory.to_str().unwrap()
        );
        if let Err(e) = fs::remove_dir_all(&self.inner.borrow().instance_directory) {
            error!("Something goes wrong {}", e)
        }
    }
}

#[allow(dead_code)]
struct Inner {
    instance_directory: PathBuf,
    resource_directory: PathBuf,
    runtime_directory: PathBuf,
    executable_directory: PathBuf,
    executable: Option<String>,
    emulator: Option<Rc<dyn Emulator>>,
    spi_map: HashMap<String, Rc<dyn Target>>,
    gpio_map: HashMap<String, Rc<dyn GpioPin>>,
    i2c_map: HashMap<String, Rc<dyn Bus>>,
    uart_map: HashMap<String, Rc<dyn Uart>>,
}

impl Inner {}

impl Transport for Ti50Emulator {
    fn capabilities(&self) -> Result<Capabilities> {
        Ok(Capabilities::new(
            Capability::UART
                | Capability::GPIO
                | Capability::SPI
                | Capability::I2C
                | Capability::EMULATOR,
        ))
    }

    // Create SPI Target instance, or return one from a cache of previously created instances.
    fn spi(&self, instance: &str) -> Result<Rc<dyn Target>> {
        Ok(Rc::new(spi::Ti50Spi::open(self, instance)?))
    }

    // Create I2C Target instance, or return one from a cache of previously created instances.
    fn i2c(&self, instance: &str) -> Result<Rc<dyn Bus>> {
        Ok(Rc::new(i2c::Ti50I2c::open(self, instance)?))
    }

    // Create Uart instance, or return one from a cache of previously created instances.
    fn uart(&self, instance: &str) -> Result<Rc<dyn Uart>> {
        Ok(Rc::new(uart::Ti50Uart::open(self, instance)?))
    }

    // Create GpioPin instance, or return one from a cache of previously created instances.
    fn gpio_pin(&self, pinname: &str) -> Result<Rc<dyn GpioPin>> {
        Ok(Rc::new(gpio::Ti50GpioPin::open(self, pinname)?))
    }

    // Create Emulator instance, or return one from a cache of previously created instances.
    fn emulator(&self) -> Result<Rc<dyn Emulator>> {
        let mut inner = self.inner.borrow_mut();
        if inner.emulator.is_none() {
            inner.emulator = Some(Rc::new(Ti50Emu::open(self)?))
        }
        Ok(Rc::clone(inner.emulator.as_ref().unwrap()))
    }
}
