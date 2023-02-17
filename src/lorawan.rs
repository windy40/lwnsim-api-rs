#![allow(unused)]

use super::lwnsim::LWNSIM;
use super::lwnsim::*;
use super::lwnsim_cmd::*;
use serde::Serialize;
use serde_json::json;

use lazy_static::lazy_static;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

// log
use log::{debug, info, trace, warn};
use anyhow::{Result};

// LoRa stack mode
pub static LORAWAN: usize = 1;
// LoRaWAN region
pub static EU868: usize = 1;
// LoRaWAN join procedure
pub static ABP: usize = 0;
pub static OTAA: usize = 1;
// LoRa triggers
pub static RX_PACKET_EVENT: usize = 1;
pub static TX_PACKET_EVENT: usize = 2;
pub static TX_FAILED_EVENT: usize = 4;
pub static JOIN_ACCEPT_EVENT: usize = 16;
pub static UNJOIN_EVENT: usize = 32;

//pub static LORA_EVENT_NAME : &'static [&'static str] = &["RX_PACKET_EVENT","TX_PACKET_EVENT","TX_FAILED_EVENT","JOIN_ACCEPT_EVENT","UNJOIN_EVENT"];

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
    last_event: usize,
    trigger: Option<usize>,
    //   handler: Option<F>,
    //   arg: Option<>,
    mode: usize,
    region: usize,
    recv_buf: Option<String>,
    error_status: usize,
}

impl Lorawan {
    fn new(mode: usize, region: usize) -> Lorawan {
        Lorawan {
            dev_eui: LWNSIM.lock().unwrap().get_dev_eui().to_string(),
            last_event: 0,
            trigger: None,
            // handler : None,
            // arg : None,
            status: LorawanStatus::JoinNOK,
            error_status: 0,
            mode: mode,
            region: region,
            recv_buf: None,
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

        self.handle_user_data(msg, SendMode::Emit);
    }

    pub fn has_joined(&self) -> bool {
        return self.status == LorawanStatus::JoinOK;
    }
    pub fn send(&mut self, mtype: &str, pl: &str) {
        let msg: DevExecuteSendUplink = DevExecuteSendUplink {
            cmd: CMD_SEND_UPLINK.to_string(),
            ack: false,
            dev_eui: self.dev_eui.clone(),
            mtype: mtype.to_string(),
            payload: pl.to_string(),
        };
        self.handle_user_data(msg, SendMode::Emit);
    }

    pub fn recv(&mut self, buffersize: usize) -> Option<String> {
        self.clear_error_status();
        self.clear_recv_buf();
        let msg: DevExecuteRecvDownlink = DevExecuteRecvDownlink {
            cmd: CMD_RECV_DOWNLINK.to_string(),
            ack: false,
            dev_eui: self.dev_eui.clone(),
            buffer_size: buffersize,
        };

        if let Ok(Some(mut resp_cmd)) = self.handle_user_data(msg, SendMode::Call) {
            if resp_cmd.get_error() != DEV_CMD_OK {
                trace!(
                    "[LORA][recv][ERROR]{:?}",
                    CMD_ERROR_NAME[resp_cmd.get_error()]
                );
                return None;
            } else {
                return resp_cmd.get_payload();
            }
        }
        return None;
    }

    fn handle_user_data(
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

    /* 	fn callback(&mut self, trigger : Option<usize>, handler : Option<>, arg : Option <>){
        self.trigger=trigger;
        self.handler=handler;
    } */

    pub fn events(&mut self) -> usize {
        let evt = self.last_event;
        self.last_event = 0;
        return evt;
    }

    pub fn handle_lora_event(&mut self, event_val: usize) {
        if event_val == JOIN_ACCEPT_EVENT {
            trace!("[LORA][Event]JoinAccept");
            self.status = LorawanStatus::JoinOK;
            return;
        } else if event_val == UNJOIN_EVENT {
            trace!("[LORA][Event]Unjoin");
            self.status = LorawanStatus::JoinNOK;
            return;
        }
        trace!("[LORA][Event]{:?}", event_val);
        self.last_event |= event_val;
        /* 		if self.last_event & self.trigger{
            trace!("[LORA][Event] triggers a callback");
            self.handler();
        } */
    }
    pub fn set_recv_buf(&mut self, data: &str) {
        self.recv_buf = Some(data.to_string());
    }
    fn get_recv_buf(&mut self) -> Option<String> {
        return self.recv_buf.take();
    }

    fn clear_recv_buf(&mut self) {
        self.recv_buf = None;
    }

    pub fn set_error_status(&mut self, error: usize) {
        self.error_status |= error;
    }

    fn get_error_status(&mut self) -> usize {
        let er = self.error_status;
        self.clear_error_status();
        return er;
    }
    fn clear_error_status(&mut self) {
        self.error_status = 0;
    }
}
