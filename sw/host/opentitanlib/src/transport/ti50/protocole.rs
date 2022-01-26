// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub enum ControlPacket {
    Req(Request),
    Res(Response),
}

#[derive(Serialize, Deserialize)]
pub enum Request {
    Status,
    Get { dev: String },
    Gpio(GpioCommand),
    Exit,
    Start(EmulatorArgs),
    Stop,
    Restart(EmulatorArgs),
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Status(Result<DutState, ErrorMessage>),
    Get(Result<DeviceEntry, ErrorMessage>),
    Gpio(Result<GpioResult, ErrorMessage>),
    Exit(Result<(), ErrorMessage>),
    Start(Result<(), ErrorMessage>),
    Stop(Result<(), ErrorMessage>),
    Restart(Result<(), ErrorMessage>),
}

#[derive(Serialize, Deserialize)]
pub enum ErrorMessage {
    ERROR(String),   // Runtime error
    INVALID(String), // Invalid argument
    BUSY,            // DUT is busy
}

#[derive(Serialize, Deserialize)]
pub enum GpioValue {
    Hi, // Logic High
    Lo, // Logic Low
    Z,  // Hi-impedance
    X,  // Undefined - random value
}

#[derive(Serialize, Deserialize)]
pub enum GpioPullMode {
    PullUp,
    PullDown,
    PullNone,
}

#[derive(Serialize, Deserialize)]
pub enum GpioMode {
    PushPull,
    OpenDrain,
    Input,
}

#[derive(Serialize, Deserialize)]
pub enum GpioCommand {
    Set { id: String, logic: bool },
    Get { id: String },
    SetMode { id: String, mode: GpioMode },
    SetPullMode { id: String, pull: GpioPullMode },
}

#[derive(Serialize, Deserialize)]
pub enum GpioResult {
    Get {
        value: GpioValue
    },
    Set,
    SetMode,
    SetPullMode,
}

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum DutState {
    PowerOn,  // in power on state
    PowerOff, // in power off state
    Busy,     // transient state representing reset
    Error,    // detected crash or runtime error
}

#[derive(Clone, Serialize, Deserialize)]
pub enum InterfaceTyp {
    UnixDatagram,
    UnixStream,
    Fifo,
    Pty,
    RegularFile,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceEntry {
    pub id: String,
    pub filename: String,
    pub typ: InterfaceTyp,
}

#[derive(Serialize, Deserialize)]
pub struct EmulatorArgs {
    pub exec: String,                 // Emulator binary
    pub args: Vec<(String, String)>, // Emulator arguments - order matter
}

