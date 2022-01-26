// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use opentitanlib::transport::ti50::Ti50;
use opentitanlib::transport::Transport;
use structopt::StructOpt;

use crate::backend::BackendOpts;

pub fn create(args: &BackendOpts) -> Result<Box<dyn Transport>> {
    Ok(Box::new(Ti50::new(args.instance_id.clone())?))
}

