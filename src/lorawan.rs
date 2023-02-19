#![allow(unused)]

use super::lora_events::LoraEvents;
use super::lwnsim::LWNSIM;
use super::lwnsim::*;
use super::lwnsim_cmd::*;
use super::error::{Error, Result};
use serde::Serialize;
use serde_json::json;

use lazy_static::lazy_static;
use std::sync::atomic::AtomicU64;
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
    pub static ref LORA: Mutex<Lorawan> = Mutex::new(Lorawan::new(LORAWAN, EU868));
}

static LORA_EVENTS: AtomicU64 = AtomicU64::new(0);

#[derive(PartialEq)]
enum LorawanStatus {
    JoinNOK,
    JoinInit,
    JoinOK,
}
pub struct Lorawan {
    status: LorawanStatus,
    dev_eui: String,
    last_event: LoraEvents,
    // trigger: Option<usize>,
    //   handler: Option<F>,
    //   arg: Option<>,
    mode: usize,
    region: usize,
}

impl Lorawan {
    fn new(mode: usize, region: usize) -> Lorawan {
        Lorawan {
            dev_eui: LWNSIM.lock().unwrap().get_dev_eui().to_string(),
            last_event: LoraEvents::NO_LORA_EVENT,
            // trigger: None,
            // handler : None,
            // arg : None,
            status: LorawanStatus::JoinNOK,
            mode: mode,
            region: region,
        }
    }

    pub fn get_dev_eui(&self) -> &str {
        return &self.dev_eui;
    }

    pub fn join(
        &mut self,
        activation: usize,
        auth: (String, String),
        timeout: Option<usize>,
        dr: Option<usize>,
    ) {
        if let Some(dur) = timeout {
            if dur != 0 {
                warn!("[LORA][join]join with timeout not implemented");
                return;
            }
        }
        trace!("[LORA][join]");
        let msg: DevExecuteCmd = DevExecuteCmd {
            cmd: CMD_JOIN_REQUEST.to_string(),
            ack: false,
            dev_eui: self.dev_eui.clone(),
        };

        self.send_lora_cmd(msg, SendMode::Emit);
    }

    pub fn has_joined(&self) -> bool {
        return self.status == LorawanStatus::JoinOK;
    }
    pub fn send(&mut self, mtype: &str, pl: &str) -> Result<()>{
        let msg: DevExecuteSendUplink = DevExecuteSendUplink {
            cmd: CMD_SEND_UPLINK.to_string(),
            ack: false,
            dev_eui: self.dev_eui.clone(),
            mtype: mtype.to_string(),
            payload: pl.to_string(),
        };

        if self.send_lora_cmd(msg, SendMode::Emit)?.is_some(){
            return Err(Error::CmdError(CmdErrorKind::UnexpectedError));
        }else{
            return Ok(());
        }
    }

// non blocking receive (for blocking receive, use lora socket) 
    pub fn recv(&mut self, buffersize: usize) -> Result<String> {
        let msg: DevExecuteRecvDownlink = DevExecuteRecvDownlink {
            cmd: CMD_RECV_DOWNLINK.to_string(),
            ack: false,
            dev_eui: self.dev_eui.clone(),
            buffer_size: buffersize,
        };

        let mut resp_cmd = self.send_lora_cmd(msg, SendMode::Call)?.unwrap();

        if resp_cmd.get_error() != CmdErrorKind::DevCmdOK {
            trace!(
                "[LORA][recv][ERROR]{:?}",
                resp_cmd.get_error()
            );
            return Err(Error::CmdError(resp_cmd.get_error()));
        } else {
            return Ok(resp_cmd.get_payload());
        }
    }

    fn send_lora_cmd(
        &self,
        msg: impl DevExecuteCmdTrait + serde::Serialize,
        mode: SendMode,
    ) -> Result<Option<Box<dyn DevResponseCmdTrait>>> {
        trace!(
            "[LORA][{:?}]{:?}",
            msg.get_cmd(),
            serde_json::to_string(&msg)
        );
        return LWNSIM.lock().unwrap().send_cmd(msg, mode);
    }

    pub fn events(&mut self) -> LoraEvents {
        let evt = self.last_event;
        self.last_event.clear();
        return evt;
    }

    pub fn handle_lora_event(&mut self, event_val: LoraEvents) {
        if LoraEvents::JOIN_ACCEPT_EVENT == event_val {
            trace!("[LORA][Event]JoinAccept");
            self.status = LorawanStatus::JoinOK;
            return;
        } else if event_val == LoraEvents::UNJOIN_EVENT {
            trace!("[LORA][Event]Unjoin");
            self.status = LorawanStatus::JoinNOK;
            return;
        }
        trace!("[LORA][Event]{:?}", event_val);
        self.last_event |= event_val;

        // for future async version ?
        /* 		if self.last_event & self.trigger{
            trace!("[LORA][Event] triggers a callback");
            self.handler();
        } */
    }

    // for future  async version ?
    /* 	fn callback(&mut self, trigger : Option<usize>, handler : Option<>, arg : Option <>){
        self.trigger=trigger;
        self.handler=handler;
    } */
}
