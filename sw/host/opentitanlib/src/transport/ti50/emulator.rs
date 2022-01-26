// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::time::Duration;

//use crate::io::emulator::Emulator;
use crate::io::emulator::{EmulatorState, EmulatorArguments, EmulatorError, Emulator};
use crate::transport::ti50::control::ControlInterface;
use crate::transport::ti50::protocole::{ControlPacket, Request, Response, ErrorMessage, DutState};
use crate::transport::ti50::transport::Ti50;

/// Represents the Ti50Emulator hardware Emulator
pub struct Ti50Emulator {
    ctl: Rc<RefCell<ControlInterface>>,
}

impl Ti50Emulator {
    pub fn open(ti: &Ti50) -> Result<Self>
    {
        Ok(Ti50Emulator {
            ctl: ti.control_interface.clone(),
        })
    }
}

impl Emulator for Ti50Emulator {
    fn Status(&self) -> Result<EmulatorState>
    {
        //let response = self.ctl.execute_command(Request::Status)?;
        if let Response::Status(result) = self.ctl.borrow_mut().execute_command(Request::Status)? {
            match result {
                Ok(status) => {
                    match status {
                        DutState::PowerOn => Ok(EmulatorState::DutPowerOn),
                        DutState::PowerOff => Ok(EmulatorState::DutPowerOff),
                        DutState::Busy => Ok(EmulatorState::DutBusy),
                        DutState::Error => Ok(EmulatorState::DutError),
                    }
                },
                Err(error) => {
                    match error {
                        ErrorMessage::ERROR(msg) => Err(EmulatorError::RuntimeError(msg).into()),
                        ErrorMessage::INVALID(msg) => Err(EmulatorError::InvalidArgument(msg).into()),
                        ErrorMessage::BUSY => Err(EmulatorError::Busy().into()),
                    }
                }
            }
        } else {
            Err(EmulatorError::RuntimeError(String::from("Expected Status recived XX")).into())
        }
    }

    fn Start(&self, args: EmulatorArguments) -> Result<()>
    {
        Ok(())
    }

    fn Stop(&self) -> Result<()>
    {
        Ok(())
    }

    fn Exit(&self) -> Result<()>
    {
        Ok(())
    }

    fn Restart(&self, update: EmulatorArguments) -> Result<()>
    {
        Ok(())
    }
}


