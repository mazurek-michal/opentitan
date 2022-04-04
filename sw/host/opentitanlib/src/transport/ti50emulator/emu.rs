// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use log::info;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::io::emu::{EmuError, EmuState, EmuValue, Emulator};
use crate::transport::ti50emulator::{Inner, Ti50Emulator};
use anyhow::{bail, Result};

pub struct Ti50Emu {
    inner: Rc<RefCell<Inner>>,
}

impl Ti50Emu {
    pub fn open(ti50: &Ti50Emulator) -> Result<Self> {
        Ok(Self {
            inner: ti50.inner.clone(),
        })
    }
}

/// A trait which represents a single GPIO pin.
impl Emulator for Ti50Emu {
    /// Simple function with return `EmuState` representing current state of Emulator instance.
    fn get_state(&self) -> Result<EmuState> {
        let inner_ref = &mut self.inner.borrow_mut();
        inner_ref.update_status()?;
        Ok(inner_ref.state)
    }

    /// Start emulator with provided arguments
    fn start(&self, factory_reset: bool, args: &HashMap<String, EmuValue>) -> Result<()> {
        let inner_ref = &mut self.inner.borrow_mut();
        inner_ref.update_status()?;
        match inner_ref.state {
            EmuState::On => {
                bail!(EmuError::StartFailureCause(String::from(
                    "DUT is already running",
                )));
            }
            EmuState::Busy => {
                bail!(EmuError::StartFailureCause(String::from(
                    "DUT is in transient state BUSY",
                )));
            }
            EmuState::Error => {
                info!("DUT trying to recover after error");
            }
            _ => {}
        }
        inner_ref.state = EmuState::Busy;
        if factory_reset {
            inner_ref.cleanup()?
        }
        inner_ref.update_args(factory_reset, args)?;
        inner_ref.spawn_process()?;
        inner_ref.state = EmuState::On;
        Ok(())
    }

    /// Stop emulator instance.
    fn stop(&self) -> Result<()> {
        let inner_ref = &mut self.inner.borrow_mut();
        inner_ref.update_status()?;
        match inner_ref.state {
            EmuState::Off => {
                bail!(EmuError::StopFailureCause(String::from(
                    "DUT is alredy Off"
                ),));
            }
            EmuState::Busy => {
                bail!(EmuError::StopFailureCause(String::from(
                    "DUT is in transient state BUSY"
                ),));
            }
            EmuState::Error => {
                info!("DUT stop after error");
            }
            _ => {}
        }
        inner_ref.stop_process()?;
        Ok(())
    }
}
