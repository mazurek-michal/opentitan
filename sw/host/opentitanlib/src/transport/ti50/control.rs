// Copyright lowRISC contributors. Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use thiserror::Error;
use log::info;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::time::Duration;
use std::os::unix::net::UnixStream;
use serde::{Deserialize, Serialize};

use crate::transport::ti50::protocole::{ControlPacket, Request, Response};

#[derive(Error, Debug)]
pub enum ControlInterfaceError {
    #[error("Invalid packet type data: {0}")]
    InvalidPacketType(String),
    #[error("Unconnected control interface")]
    Unconnected,
}

pub struct ControlInterface {
    name: PathBuf,
    stream: Option<UnixStream>,
}

impl ControlInterface {

    pub fn connect(instance_name: &String) -> Result<Self>
    {
        let mut path = PathBuf::from("/tmp");
        path.push(instance_name);
        path.push("ctl.unix");
        info!("Ti50 ControlInterface connecting: {}", path.to_str().unwrap_or("invalid"));
        let socket = UnixStream::connect(&path)?;
        socket.set_read_timeout(Some(Duration::from_millis(5000)))?;
        socket.set_write_timeout(Some(Duration::from_millis(5000)))?;
        Ok(ControlInterface {
            name: path.clone(),
            stream: Some(socket)
        })
    }

    //think about timeout
    pub fn execute_command(&mut self, request: Request) -> Result<Response>
    {
        if let Some(connection) = &mut self.stream {
            //send request
            let mut data = serde_json::to_vec(&ControlPacket::Req(request))?;
            data.push('\n' as u8);
            connection.write(data.as_slice())?;
            //wait for data
            let mut buffer = String::new();
            connection.read_to_string(&mut buffer)?;
            let packet: ControlPacket = serde_json::from_str(buffer.as_str())?;
            return match packet {
                ControlPacket::Res(resp) => Ok(resp),
                ControlPacket::Req(_) => Err(ControlInterfaceError::InvalidPacketType(buffer).into()),
            };
        }
        Err(ControlInterfaceError::Unconnected.into())
    }

}
