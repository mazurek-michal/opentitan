// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use crate::io::i2c::{Bus, Transfer};
use crate::transport::ti50emulator::Ti50Emulator;
use crate::transport::{Result, TransportError};

pub struct Ti50I2c {}

// FIXME: remove 'dead_code' after implementing I2C
#[allow(dead_code)]
impl Ti50I2c {
    pub fn open(_emulator: &Ti50Emulator, _instance: &str) -> Result<Self> {
        Err(TransportError::UnsupportedOperation.into())
    }
}

impl Bus for Ti50I2c {
    fn run_transaction(&self, _addr: u8, _transaction: &mut [Transfer]) -> Result<()> {
        Err(TransportError::UnsupportedOperation.into())
    }
}
