// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use anyhow::Result;
use serde::Deserialize;
use structopt::clap::arg_enum;
use thiserror::Error;

pub enum EmulatorState {
    DutPowerOn,
    DutPowerOff,
    DutBusy,
    DutError,
}

#[derive(Debug, Error)]
pub enum EmulatorError {
    #[error("Runtime Error {0}")]
    RuntimeError(String),
    #[error("Invalid Argument {0}")]
    InvalidArgument(String),
    #[error("Emulator in Busy state")]
    Busy(),
}

pub struct EmulatorArguments{
    executable: String,
    arguments: HashMap<String, String>,
}

pub trait Emulator {
    fn Status(&self) -> Result<EmulatorState>;
    fn Start(&self, args: EmulatorArguments) -> Result<()>;
    fn Stop(&self) -> Result<()>;
    fn Exit(&self) -> Result<()>;
    fn Restart(&self, update: EmulatorArguments) -> Result<()>;
}

