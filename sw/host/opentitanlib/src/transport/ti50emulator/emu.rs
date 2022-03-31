// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;
use std::collections::HashMap;
use std::process::Child;
use std::rc::Rc;

use crate::io::emu::{EmuState, EmuValue, Emulator};
use crate::transport::ti50emulator::{Inner, Ti50Emulator};
use crate::transport::{Result, TransportError};

#[allow(dead_code)]
pub struct Ti50Emu {
    inner: Rc<RefCell<Inner>>,
    current_args: HashMap<String, EmuValue>,
    executable: Option<String>,
    state: EmuState,
    proc: Option<Child>,
}

impl Ti50Emu {
    pub fn open(ti50: &Ti50Emulator) -> Result<Self> {
        Ok(Self {
            inner: ti50.inner.clone(),
            current_args: HashMap::new(),
            executable: None,
            state: EmuState::Off,
            proc: None,
        })
    }
}

/// A trait which represents a single GPIO pin.
impl Emulator for Ti50Emu {
    /// Simple function with return `EmuState` representing current state of Emulator instance.
    fn get_state(&self) -> Result<EmuState> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Start emulator with provided arguments
    fn start(&self, _factory_reset: bool, _args: &HashMap<String, EmuValue>) -> Result<()> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Stop emulator instance.
    fn stop(&self) -> Result<()> {
        Err(TransportError::UnsupportedOperation.into())
    }
}
