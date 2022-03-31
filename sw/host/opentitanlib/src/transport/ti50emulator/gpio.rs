// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use crate::io::gpio::{GpioPin, PinMode, PullMode};
use crate::transport::ti50emulator::Ti50Emulator;
use crate::transport::{Result, TransportError};

pub struct Ti50GpioPin {}

impl Ti50GpioPin {
    pub fn open(_emulator: &Ti50Emulator, _pinname: &str) -> Result<Self> {
        Err(TransportError::UnsupportedOperation.into())
    }
}

/// A trait which represents a single GPIO pin.
impl GpioPin for Ti50GpioPin {
    /// Reads the value of the the GPIO pin.
    fn read(&self) -> Result<bool> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Sets the value of the GPIO pin to `value`.
    fn write(&self, _value: bool) -> Result<()> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Sets the mode of the GPIO pin as input, output, or open drain I/O.
    fn set_mode(&self, _mode: PinMode) -> Result<()> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Sets the weak pull resistors of the GPIO pin.
    fn set_pull_mode(&self, _mode: PullMode) -> Result<()> {
        Err(TransportError::UnsupportedOperation.into())
    }
}
