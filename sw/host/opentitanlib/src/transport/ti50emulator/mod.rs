// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Context, Result};
use log::{error, info};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{ErrorKind, Read};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::process;
use std::process::{Child, Command};
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::io::emu::{EmuError, EmuState, EmuValue, Emulator};
use crate::io::gpio::GpioPin;
use crate::io::i2c::Bus;
use crate::io::spi::Target;
use crate::io::uart::Uart;
use crate::transport::{
    Capabilities, Capability, Transport, TransportError, TransportInterfaceType,
};

mod emu;
mod gpio;
mod i2c;
mod spi;
mod uart;

use crate::transport::ti50emulator::emu::Ti50Emu;

const TIMEOUT: Duration = Duration::from_millis(100);
const MAX_RETRY: usize = 3;

/// Implementation of the Transport trait backed by connection to a remote OpenTitan tool
/// session process.
pub struct Ti50Emulator {
    inner: Rc<RefCell<Inner>>,
}

impl Ti50Emulator {
    /// Establish connection with a running session process.
    pub fn open(
        executable_directory: PathBuf,
        executable: &String,
        instance_prefix: &String,
    ) -> anyhow::Result<Self> {
        let tstamp = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let instance_name = format!(
            "{}_{}_{}_{}",
            instance_prefix,
            process::id(),
            tstamp.as_secs(),
            tstamp.as_nanos()
        );

        let mut instance_directory = PathBuf::from("/tmp");
        instance_directory.push(&instance_name);
        let resource_directory = instance_directory.join("resources");
        let runtime_directory = instance_directory.join("runtime");

        info!("Initializing Ti50Emulator instance:{}", instance_name);
        fs::create_dir(&instance_directory)?;
        fs::create_dir(&resource_directory)?;
        fs::create_dir(&runtime_directory)?;

        Ok(Self {
            inner: Rc::new(RefCell::new(Inner {
                instance_directory: instance_directory.clone(),
                resource_directory: resource_directory.clone(),
                runtime_directory: runtime_directory.clone(),
                executable_directory: executable_directory,
                executable: Some(executable.clone()),
                current_args: HashMap::new(),
                state: EmuState::Off,
                proc: None,
                emulator: None,
                spi_map: HashMap::new(),
                gpio_map: HashMap::new(),
                i2c_map: HashMap::new(),
                uart_map: HashMap::new(),
            })),
        })
    }
}

impl Drop for Ti50Emulator {
    fn drop(&mut self) {
        info!(
            "Clenup Ti50Emulator instance directory:{}",
            self.inner.borrow().instance_directory.to_str().unwrap()
        );
        if let Err(e) = fs::remove_dir_all(&self.inner.borrow().instance_directory) {
            error!("Something goes wrong {}", e)
        }
    }
}

// FIXME: remove 'dead_code' after implementing resource management
#[allow(dead_code)]
struct Inner {
    instance_directory: PathBuf,
    resource_directory: PathBuf,
    runtime_directory: PathBuf,
    executable_directory: PathBuf,
    executable: Option<String>,

    current_args: HashMap<String, EmuValue>,
    state: EmuState,
    proc: Option<Child>,

    emulator: Option<Rc<dyn Emulator>>,
    spi_map: HashMap<String, Rc<dyn Target>>,
    gpio_map: HashMap<String, Rc<dyn GpioPin>>,
    i2c_map: HashMap<String, Rc<dyn Bus>>,
    uart_map: HashMap<String, Rc<dyn Uart>>,
}

impl Inner {
    fn update_status(&mut self) -> Result<()> {
        if let Some(proc) = &mut self.proc {
            match proc.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        info!("Ti50Emulator exit with status {}", status);
                        self.state = EmuState::Off;
                    } else {
                        if self.state != EmuState::Error {
                            info!("Ti50Emualtor sub-process exit with error: {}", status)
                        }
                        self.state = EmuState::Error;
                    }
                    self.proc = None;
                }
                Ok(None) => {
                    self.state = EmuState::On;
                }
                Err(err) => {
                    bail!(EmuError::RuntimeError(format!(
                        "Can't aquire status from process pid:{} error:{}",
                        proc.id(),
                        err
                    )));
                }
            }
        } else {
            if self.state == EmuState::On {
                self.state = EmuState::Error;
                bail!(EmuError::RuntimeError(
                    "Non sub-process found but state indicate that Emulator is ON".to_string()
                ));
            }
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        fs::remove_dir_all(&self.runtime_directory)?;
        fs::create_dir(&self.runtime_directory)?;
        Ok(())
    }

    fn spawn_process(&mut self) -> Result<()> {
        let socket_path = self.runtime_directory.join("control_soc");
        let control_socket = UnixListener::bind(socket_path.clone())?;
        control_socket.set_nonblocking(true)?;
        self.current_args.insert(
            "path".to_string(),
            EmuValue::FilePath(self.runtime_directory.to_string_lossy().to_string()),
        );
        self.current_args.insert(
            "control_socket".to_string(),
            EmuValue::FilePath(socket_path.to_string_lossy().to_string()),
        );
        let mut args_list: Vec<String> = Vec::new();
        for (key, item) in self.current_args.iter() {
            match item {
                EmuValue::Empty => {
                    args_list.push(format!("--{}", key));
                }
                EmuValue::String(value) | EmuValue::FilePath(value) => {
                    args_list.push(format!("--{} {}", key, value));
                }
                EmuValue::StringList(value_array) => {
                    args_list.push(format!("--{} {}", key, value_array.join(",")));
                }
                EmuValue::FilePathList(value_array) => {
                    args_list.push(format!("--{} {}", key, value_array.join(",")));
                }
            }
        }
        let exec = self
            .executable_directory
            .join(self.executable.clone().unwrap());
        info!("Spawning Ti50Emulator sub-process");
        info!("Command: {}", args_list.join(" "));
        let mut cmd = Command::new(exec);
        match cmd.spawn() {
            Ok(handle) => {
                self.proc = Some(handle);
                let pattern = b"READY";
                let mut buffer = [0u8, 8];
                let mut retry = 0;
                while retry < MAX_RETRY {
                    match control_socket.accept() {
                        Ok((mut socket, _addres)) => {
                            let len = socket.read(&mut buffer)?;
                            if len >= pattern.len() && pattern[..] == buffer[0..pattern.len()] {
                                info!("Ti50Emulator ready");
                                return Ok(());
                            }
                        }
                        Err(err) if err.kind() == ErrorKind::WouldBlock => {
                            std::thread::sleep(TIMEOUT);
                        }
                        Err(err) => {
                            self.state = EmuState::Error;
                            bail!(EmuError::StartFailureCause(format!(
                                "Can't connect to other end of sub-proces control socket error:{}",
                                err
                            )));
                        }
                    }
                    retry += 1;
                }
                bail!(EmuError::StartFailureCause(
                    "Timeout during wiating on sub-process".to_string(),
                ));
            }
            Err(_) => {
                bail!(EmuError::StartFailureCause(String::from(
                    "DUT is in transient state BUSY"
                ),));
            }
        }
    }

    fn stop_process(&mut self) -> Result<()> {
        // Try terminate process gracefully with SIGTERM, if this method fail use SIGKILL.
        if let Some(handle) = &self.proc {
            let pid = handle.id() as i32;
            signal::kill(Pid::from_raw(pid), Signal::SIGTERM).context("Stop process")?;
            for _retry in 0..MAX_RETRY {
                std::thread::sleep(TIMEOUT);
                match signal::kill(Pid::from_raw(pid), None) {
                    Ok(()) => {}
                    Err(nix::Error::Sys(nix::errno::Errno::ESRCH)) => {
                        self.state = EmuState::Off;
                        self.proc = None;
                        return Ok(());
                    }
                    Err(e) => {
                        self.state = EmuState::Error;
                        return Err(EmuError::StopFailureCause(format!(
                            "Unexpected error querying process presence: {}",
                            e
                        ))
                        .into());
                    }
                }
            }
            // Fallback path with use SIGKILL to end process live
            signal::kill(Pid::from_raw(pid), Signal::SIGKILL).context("Stop process - fallback")?;
            std::thread::sleep(TIMEOUT);
            match signal::kill(Pid::from_raw(pid), None) {
                Err(nix::Error::Sys(nix::errno::Errno::ESRCH)) => {
                    self.proc = None;
                    self.state = EmuState::Off;
                    return Ok(());
                }
                _ => {
                    self.proc = None;
                    self.state = EmuState::Error;
                    return Err(EmuError::StopFailureCause(format!(
                        "Unable to stop process pid:{}",
                        pid
                    ))
                    .into());
                }
            }
        }
        Ok(())
    }

    fn update_args(
        &mut self,
        _factory_reset: bool,
        args: &HashMap<String, EmuValue>,
    ) -> Result<()> {
        let forbiden = HashSet::from([
            "p".to_string(),
            "path".to_string(),
            "s".to_string(),
            "stdio".to_string(),
            "control_socket".to_string(),
        ]);
        let allowed = HashSet::from([
            "flash".to_string(),
            "apps".to_string(),
            "version_state".to_string(),
            "pmu_state".to_string(),
        ]);
        for (key, item) in args.iter() {
            if forbiden.contains(key) {
                bail!(EmuError::InvalidArgumetName(key.clone(),));
            }
            if allowed.contains(key) {
                self.current_args.insert(key.clone(), item.clone());
                continue;
            }
            //TODO add resource mangment for factory reset
            //TODO add value validations
            bail!(EmuError::InvalidArgumetName(key.clone()));
        }
        Ok(())
    }
}

impl Transport for Ti50Emulator {
    fn capabilities(&self) -> Result<Capabilities> {
        Ok(Capabilities::new(
            Capability::UART
                | Capability::GPIO
                | Capability::SPI
                | Capability::I2C
                | Capability::EMULATOR,
        ))
    }

    fn spi(&self, instance: &str) -> Result<Rc<dyn Target>> {
        let inner = self.inner.borrow();
        if let Some(spi) = inner.spi_map.get(instance) {
            return Ok(Rc::clone(spi));
        }
        Err(
            TransportError::InvalidInstance(TransportInterfaceType::Spi, instance.to_string())
                .into(),
        )
    }

    fn i2c(&self, instance: &str) -> Result<Rc<dyn Bus>> {
        let inner = self.inner.borrow();
        if let Some(i2c) = inner.i2c_map.get(instance) {
            return Ok(Rc::clone(i2c));
        }
        Err(
            TransportError::InvalidInstance(TransportInterfaceType::I2c, instance.to_string())
                .into(),
        )
    }

    fn uart(&self, instance: &str) -> Result<Rc<dyn Uart>> {
        let inner = self.inner.borrow();
        if let Some(uart) = inner.uart_map.get(instance) {
            return Ok(Rc::clone(uart));
        }
        Err(
            TransportError::InvalidInstance(TransportInterfaceType::Uart, instance.to_string())
                .into(),
        )
    }

    fn gpio_pin(&self, pinname: &str) -> Result<Rc<dyn GpioPin>> {
        let inner = self.inner.borrow();
        if let Some(gpio) = inner.gpio_map.get(pinname) {
            return Ok(Rc::clone(gpio));
        }
        Err(
            TransportError::InvalidInstance(TransportInterfaceType::Gpio, pinname.to_string())
                .into(),
        )
    }

    fn emulator(&self) -> Result<Rc<dyn Emulator>> {
        let mut inner = self.inner.borrow_mut();
        if inner.emulator.is_none() {
            inner.emulator = Some(Rc::new(Ti50Emu::open(self)?))
        }
        Ok(Rc::clone(inner.emulator.as_ref().unwrap()))
    }
}
