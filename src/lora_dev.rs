#![allow(unused)]

use super::error::{Error, Result};
use super::lora_events::{LORA_EVENTS, LoraEvents};
use super::lwnsim::LWNSIM;
use super::lwnsim::*;
use super::lwnsim_cmd::*;
use serde::Serialize;
use serde_json::json;

use lazy_static::lazy_static;
use std::sync::Mutex;

// log
use log::{debug, info, trace, warn};

// LoRa stack mode
pub static LORAWAN: usize = 1;
// LoRaWAN region
pub static EU868: usize = 1;
// LoRaWAN join procedure
pub static ABP: usize = 0;
pub static OTAA: usize = 1;

lazy_static! {
    pub static ref LORA: Mutex<LoraDev> = Mutex::new(LoraDev::new(LORAWAN, EU868));
}

#[derive(PartialEq)]
pub enum LoraDevStatus {
    Inactive,
    Active,
    Joined,
    Unjoined,
}
pub struct LoraDev {
    status: LoraDevStatus,
    dev_eui: String,
    // lora_events: LoraEvents,
    // trigger: Option<usize>,
    //   handler: Option<F>,
    //   arg: Option<>,
    mode: usize,
    region: usize,
}

impl LoraDev {
    fn new(mode: usize, region: usize) -> LoraDev {
        LoraDev {
            dev_eui: LWNSIM.lock().unwrap().get_dev_eui().to_string(),
            //lora_events: LoraEvents::NO_LORA_EVENT,
            // trigger: None,
            // handler : None,
            // arg : None,
            status: LoraDevStatus::Inactive,
            mode: mode,
            region: region,
        }
    }

    pub fn get_dev_eui(&self) -> &str {
        return &self.dev_eui;
    }

    pub fn set_status(&mut self, status: LoraDevStatus) {
        self.status = status;
    }
    pub fn activate(&mut self) -> Result<()> {
        if self.status == LoraDevStatus::Inactive {
            trace!("[LORA][activate]");
            let msg = DevExecuteCmd {
                cmd: CMD_LINK_DEV.to_string(),
                ack: true,
                dev_eui: self.get_dev_eui().to_string(),
            };
            match self
                .send_lora_cmd(msg, SendMode::Call)?
                .unwrap()
                .get_error()
            {
                CmdErrorKind::DevCmdOK => {
                    self.set_status(LoraDevStatus::Active);
                    info!("[LORA][activate]OK");
                    return Ok(());
                }
                k => {
                    self.set_status(LoraDevStatus::Inactive);
                    return Err(Error::CmdError(k));
                }
            }
        } else {
            return Err(Error::CmdError(CmdErrorKind::DeviceLinked));
        }
    }

    pub fn unlink_dev(&mut self) -> Result<()> {
        if self.status != LoraDevStatus::Inactive {
            let msg = DevExecuteCmd {
                cmd: CMD_UNLINK_DEV.to_string(),
                ack: true,
                dev_eui: self.get_dev_eui().to_string(),
            };
            match self
                .send_lora_cmd(msg, SendMode::Call)?
                .unwrap()
                .get_error()
            {
                CmdErrorKind::DevCmdOK => {
                    self.set_status(LoraDevStatus::Inactive);
                    info!("[LORA][unlink_dev]OK");
                    return Ok(());
                }
                k =>
                // self.set_status(); device status unchanged
                {
                    return Err(Error::CmdError(k))
                }
            }
        } else {
            return Err(Error::CmdError(CmdErrorKind::DeviceNotLinked));
        }
    }

    pub fn join(
        &mut self,
        activation: usize,
        auth: (String, String),
        timeout: Option<usize>,
        dr: Option<usize>,
    ) -> Result<()> {
        if let Some(dur) = timeout {
            if dur != 0 {
                warn!("[LORA][join]join with timeout not implemented");
                return Err(Error::CmdError(CmdErrorKind::NIY));
            }
        }
        if self.status == LoraDevStatus::Active || self.status == LoraDevStatus::Unjoined {
            info!("[LORA][join]start");
            let msg: DevExecuteCmd = DevExecuteCmd {
                cmd: CMD_JOIN_REQUEST.to_string(),
                ack: false,
                dev_eui: self.dev_eui.clone(),
            };

            return self.send_lora_cmd(msg, SendMode::Emit).map(|_| ());
        } else {
            return Err(Error::CmdError(CmdErrorKind::DeviceNotJoined));
        }
    }

    pub fn has_joined(&mut self) -> bool {
        if LORA_EVENTS.lock().unwrap().contains(LoraEvents::JOIN_ACCEPT_EVENT){
            self.set_status(LoraDevStatus::Joined);
            true
        }else {
            false
        }
    }
    pub fn send(&mut self, mtype: &str, pl: &str) -> Result<()> {
        if self.status == LoraDevStatus::Joined {
            let msg: DevExecuteSendUplink = DevExecuteSendUplink {
                cmd: CMD_SEND_UPLINK.to_string(),
                ack: false,
                dev_eui: self.dev_eui.clone(),
                mtype: mtype.to_string(),
                payload: pl.to_string(),
            };

            return self.send_lora_cmd(msg, SendMode::Emit).map(|_| ());
        } else {
            return Err(Error::CmdError(CmdErrorKind::DeviceNotJoined));
        }
    }

    // non blocking receive (for blocking receive, use lora socket)
    pub fn recv(&mut self, buffersize: usize) -> Result<String> {
        if self.status == LoraDevStatus::Joined {
            let msg: DevExecuteRecvDownlink = DevExecuteRecvDownlink {
                cmd: CMD_RECV_DOWNLINK.to_string(),
                ack: false,
                dev_eui: self.dev_eui.clone(),
                buffer_size: buffersize,
            };

            let mut resp_cmd = self.send_lora_cmd(msg, SendMode::Call)?.unwrap();

            if resp_cmd.get_error() != CmdErrorKind::DevCmdOK {
                trace!("[LORA][recv][ERROR]{:?}", resp_cmd.get_error());
                return Err(Error::CmdError(resp_cmd.get_error()));
            } else {
                return Ok(resp_cmd.get_payload());
            }
        } else {
            return Err(Error::CmdError(CmdErrorKind::DeviceNotJoined));
        }
    }

    fn send_lora_cmd(
        &mut self,
        msg: impl DevExecuteCmdTrait + serde::Serialize,
        mode: SendMode,
    ) -> Result<Option<Box<dyn ResponseCmdTrait>>> {
        trace!(
            "[LORA][{:?}]{:?}",
            msg.get_cmd(),
            serde_json::to_string(&msg)
        );
        let cmd_resp = LWNSIM.lock().unwrap().send_cmd(msg, mode);
        match cmd_resp {
            Err(Error::CmdError(CmdErrorKind::SimulatorNotRunning))
            | Err(Error::CmdError(CmdErrorKind::NoDeviceWithDevEUI)) => {
                self.status = LoraDevStatus::Inactive;
                return cmd_resp;
            }
            _ => return cmd_resp,
        }
    }

    /*     pub fn events(&mut self) -> LoraEvents {
        let evt = self.lora_events;
        self.lora_events.clear();
        return evt;
    }

    pub fn get_events(& self) -> LoraEvents {
        return self.lora_events;
    }

    pub fn set_event(&mut self, evt : LoraEvents) {
        self.lora_events |= evt;
    }

    pub fn clear_event(&mut self, evt : LoraEvents) {
        self.lora_events &= !evt;
    }

    pub fn handle_lora_event(&mut self, event_val: LoraEvents) {
        if LoraEvents::JOIN_ACCEPT_EVENT == event_val {
            trace!("[LORA][Event]JoinAccept");
            self.status = LoraDevStatus::Joined;
            return;
        } else if event_val == LoraEvents::UNJOIN_EVENT {
            trace!("[LORA][Event]Unjoin");
            self.status = LoraDevStatus::Unjoined;
            return;
        }
        // set TX and RX type events
        // these events are used for blocking send and receive and possibly for async version in the futures
        trace!("[LORA][Event]{:?}", event_val);
        self.set_event(event_val);

        // for future async version ?
        //  		if self.lora_events & self.trigger{
        //     trace!("[LORA][Event] triggers a callback");
        //     self.handler();
        // }
    }
    */

    // for future  async version ?
    /* 	fn callback(&mut self, trigger : Option<usize>, handler : Option<>, arg : Option <>){
        self.trigger=trigger;
        self.handler=handler;
    } */
}
