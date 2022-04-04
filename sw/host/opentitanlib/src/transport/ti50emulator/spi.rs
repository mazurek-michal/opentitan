// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use crate::io::spi::{SpiError, Target, Transfer, TransferMode};
use crate::transport::ti50emulator::Ti50Emulator;
use crate::transport::{Result, TransportError};
use crate::util::voltage::Voltage;

pub struct Ti50Spi {}

// FIXME: remove 'dead_code' after implementing SPI
#[allow(dead_code)]
impl Ti50Spi {
    pub fn open(_emulator: &Ti50Emulator, _instance: &str) -> Result<Self> {
        Err(TransportError::UnsupportedOperation.into())
    }
}

impl Target for Ti50Spi {
    /// Gets the current SPI transfer mode.
    fn get_transfer_mode(&self) -> Result<TransferMode> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Sets the current SPI transfer mode.
    fn set_transfer_mode(&self, _mode: TransferMode) -> Result<()> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Gets the current number of bits per word.
    fn get_bits_per_word(&self) -> Result<u32> {
        Ok(8)
    }

    /// Sets the current number of bits per word.
    fn set_bits_per_word(&self, _bits_per_word: u32) -> Result<()> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Gets the maximum allowed speed of the SPI bus.
    fn get_max_speed(&self) -> Result<u32> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Sets the maximum allowed speed of the SPI bus.
    fn set_max_speed(&self, _max_speed: u32) -> Result<()> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Returns the maximum number of transfers allowed in a single transaction.
    fn get_max_transfer_count(&self) -> Result<usize> {
        Err(TransportError::UnsupportedOperation.into())
    }

    /// Maximum chunksize handled by this SPI device.
    fn max_chunk_size(&self) -> Result<usize> {
        Err(TransportError::UnsupportedOperation.into())
    }

    fn set_voltage(&self, _voltage: Voltage) -> Result<()> {
        Err(SpiError::InvalidOption("This target does not support set_voltage".to_string()).into())
    }

    /// Runs a SPI transaction composed from the slice of [`Transfer`] objects.
    fn run_transaction(&self, _transaction: &mut [Transfer]) -> Result<()> {
        Err(TransportError::UnsupportedOperation.into())
    }
}
