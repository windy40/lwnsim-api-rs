#![allow(unused)]

use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use chrono::{DateTime, Utc};
use std::time::Duration;

use lazy_static::lazy_static;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

use super::lorawan::LORA;
use super::lwnsim_cmd::*;
use super::lwnsim_cmd::{
    CMD_JOIN_REQUEST, CMD_LINK_DEV, CMD_RECV_DOWNLINK, CMD_SEND_UPLINK, CMD_UNLINK_DEV,
};
// log
use log::{debug, info, trace, warn};

use anyhow::{anyhow, Result};
//use std::error;
pub enum LwnsimStatus {
    ConnNOK,
    ConnInit,
    ConnOK,
    ConnLinkDevNOK,
    ConnLinkDevInit,
    ConnLinkDevOK,
    ConnLost,
    ConnUnlinkDevInit,
    ConnUnlinkDevOK,
    ConnUnlinkDevNOK,
    ConnDisInit,
}

// socket.io events
static DEV_EVENT_LOG: &str = "dev-log";
static DEV_EVENT_ERROR: &str = "dev-error";

static DEV_EVENT_RESPONSE_CMD: &str = "response-cmd";
static DEV_EVENT_ACK_CMD: &str = "ack-cmd";
static DEV_EVENT_LORA: &str = "lora-event";

static DEV_EVENT_LINK_DEV: &str = "link-dev";
static DEV_EVENT_UNLINK_DEV: &str = "unlink-dev";
static DEV_EVENT_JOIN_REQUEST: &str = "join-request";
static DEV_EVENT_SEND_UPLINK: &str = "send-uplink";
static DEV_EVENT_RECV_DOWNLINK: &str = "recv-downlink";

lazy_static! {
    pub static ref LWNSIM: Mutex<Lwnsim> = Mutex::new(Lwnsim::new());
}

pub enum SendMode {
    Call,
    Emit,
}

pub struct Lwnsim {
    socket: Option<Client>,
    status: LwnsimStatus,
    ack_cmd: bool,
    dev_eui: String,
    //    handle_response_cmd: Box<dyn FnMut(Payload,RawClient)->()>,
}

impl Lwnsim {
    pub fn new() -> Lwnsim {
        Lwnsim {
            status: LwnsimStatus::ConnNOK,
            socket: None,
            ack_cmd: true,
            dev_eui: "".to_string(),
        }
    }

    pub fn set_dev_eui(&mut self, dev_eui: String) {
        self.dev_eui = dev_eui;
    }

    pub fn get_dev_eui(&self) -> &str {
        return &self.dev_eui;
    }

    pub fn set_status(&mut self, status: LwnsimStatus) {
        self.status = status;
    }

    pub fn connect(&mut self, url: &str) {
        self.status = LwnsimStatus::ConnInit;

        let socket = ClientBuilder::new(url.to_string())
            .namespace("/dev")
            .on("open", |_, _| info!("Connected"))
            .on("close", |_, _| info!("Disconnected !"))
            .on(DEV_EVENT_ACK_CMD, |payload, _: RawClient| {
                if let Payload::String(s) = payload {
                    trace!("[LWNSIM][CMD_ACK][cmd]{:?}", s);
                };
            })
            .on(DEV_EVENT_LORA, |payload, _: RawClient| {
                if let Payload::String(json_str) = payload {
                    trace!("[LWNSIM][LORA EVENT]{:?}", json_str);
                    let object: Value = serde_json::from_str(&json_str).unwrap();
                    if let Value::Number(nb) = &object["event"] {
                        let mut event_id: usize = nb.as_u64().unwrap() as usize;
                        LWNSIM.lock().unwrap().push_lora_event(event_id);
                    } else {
                        warn!("[LWNSIM] DevResponseCmd json error");
                    }
                } else {
                    warn!("[LWNSIM] Payload error : not the String variant");
                }
            })
            // .on("error", |err, _| eprintln!("Error: {:#?}", err))
            .opening_header("accept-encoding", "application/json")
            .connect()
            .expect("Connection failed");
        self.socket = Some(socket);
        self.status = LwnsimStatus::ConnOK;
    }

    pub fn disconnect(&self) {
        if let Some(s) = &self.socket {
            trace!("[LWNSIM] disconnecting");
            s.disconnect().expect("Disconnect failed");
        }
    }

    pub fn link_dev(&mut self) {
        let msg = DevExecuteCmd {
            cmd: CMD_LINK_DEV.to_string(),
            ack: true,
            dev_eui: (&self.dev_eui).to_string(),
        };
        self.status = LwnsimStatus::ConnLinkDevInit;
        if let Ok(Some(cmd_resp)) = self.send_cmd(msg, SendMode::Call) {
            if cmd_resp.get_error() == DEV_CMD_OK {
                self.set_status(LwnsimStatus::ConnLinkDevOK);
            } else {
                self.set_status(LwnsimStatus::ConnLinkDevNOK);
            }
        } else {
        }
    }

    pub fn unlink_dev(&mut self) {
        let msg = DevExecuteCmd {
            cmd: CMD_UNLINK_DEV.to_string(),
            ack: true,
            dev_eui: (&self.dev_eui).to_string(),
        };
        self.status = LwnsimStatus::ConnUnlinkDevInit;

        if let Ok(Some(cmd_resp)) = self.send_cmd(msg, SendMode::Call) {
            if cmd_resp.get_error() == DEV_CMD_OK {
                self.set_status(LwnsimStatus::ConnUnlinkDevOK);
            } else {
                self.set_status(LwnsimStatus::ConnUnlinkDevNOK);
            }
        } else {
        }
    }

    pub fn send_cmd(
        &mut self,
        mut msg: impl DevExecuteCmdTrait + serde::Serialize,
        mode: SendMode,
    ) -> Result<Option<Box<dyn DevResponseCmdTrait>>> {
        if self.ack_cmd {
            msg.set_ack(true);
        }
        let msg_json = serde_json::to_value(&msg).expect("serialization to value failed");

        match mode {
            SendMode::Emit => {
                trace!(
                    "[LWNSIM][CMD_EMIT][{}]{:?}",
                    msg.get_cmd(),
                    serde_json::to_string(&msg)
                );
                self.socket
                    .as_ref()
                    .expect("socket unset")
                    .emit(msg.get_cmd(), msg_json)?;
                //                    .expect("emit failed");
                return Ok(None);
            }
            SendMode::Call => {
                let (tx, rx): (Sender<Payload>, Receiver<Payload>) = channel();
                trace!(
                    "[LWNSIM][CMD_CALL][{}]{:?}",
                    msg.get_cmd(),
                    serde_json::to_string(&msg)
                );
                self.socket
                    .as_ref()
                    .expect("socket undefined")
                    .emit_with_ack(
                        msg.get_cmd(),
                        msg_json,
                        Duration::from_secs(2),
                        move |message: Payload, _: RawClient| {
                            trace!("[LWNSIM][CMD_RESP]{:?}", message);
                            // send the result to the channel
                            tx.send(message).unwrap();
                        },
                    )?;
                //                    .expect("emit_with_ack failed or timed out");
                if let Payload::String(message) = rx.recv()? {
                    let resp_cmd: Box<dyn DevResponseCmdTrait> = serde_json::from_str(&message)?;
                    if resp_cmd.get_error() == DEV_ERROR_SIMULATOR_NOT_RUNNING
                        || resp_cmd.get_error() == DEV_ERROR_NO_DEVICE_WITH_DEVEUI
                    {
                        self.set_status(LwnsimStatus::ConnUnlinkDevOK); //automatically unlinked when simulator is stopped
                        self.disconnect();
                        //       sys.exit()
                    }
                    return Ok(Some(resp_cmd));
                }else{
                    return Err(anyhow!("[LWNSIM][PayloadParseError] not the String variant"));
                }
            }
        }
    }

    pub fn parse_resp(&mut self, payload: Payload) -> Result<Box<dyn DevResponseCmdTrait>> {
        if let Payload::String(json_str) = payload {
            let object: Value = serde_json::from_str(&json_str).unwrap();
            let mut cmd_name: &str;
            if let Value::String(tmp) = &object[0]["cmd"] {
                cmd_name = tmp;
            } else {
                warn!("[LWNSIM][Parse DevResponseCmd] DevResponseCmd json error");
                return Err(anyhow!("[Parse DevResponseCmd error] json error"));
            }
            let mut error_id: usize;
            if let Value::Number(nb) = &object[0]["error"]
            /* &*object.get("error").unwrap()*/
            {
                error_id = nb.as_u64().unwrap() as usize;
            /*
            if error_id == 0 {
                self.handle_ok_resp(cmd_name, error_id, &object);
            } else {
                self.handle_error_resp(cmd_name, error_id);
            } */
            } else {
                warn!("[LWNSIM][Parse DevResponseCmd] DevResponseCmd json error");
                return Err(anyhow!("[[Parse DevResponseCmd error] json"));
            }
            if cmd_name != CMD_RECV_DOWNLINK {
                return Ok(Box::new(DevResponseCmd {
                    cmd: cmd_name.to_string(),
                    error: error_id,
                }));
            } else {
                let mut mtype: &str;
                if let Value::String(tmp) = &object[0]["cmd"] {
                    mtype = tmp;
                } else {
                    warn!("[LWNSIM][Parse DevResponseCmd] DevResponseCmd json error");
                    return Err(anyhow!("[Parse DevResponseCmd error] json error"));
                }
                let mut payload: &str;
                if let Value::String(tmp) = &object[0]["cmd"] {
                    payload = tmp;
                } else {
                    warn!("[LWNSIM][Parse DevResponseCmd] DevResponseCmd json error");
                    return Err(anyhow!("[Parse DevResponseCmd error] json error"));
                }
                return Ok(Box::new(DevResponseRecvDownlinkCmd {
                    cmd: cmd_name.to_string(),
                    error: error_id,
                    mtype: mtype.to_string(),
                    payload: Some(payload.to_string()),
                }));
            }
        } else {
            warn!("[LWNSIM][Parse DevResponseCmd] error :Payload not the String variant");
            return Err(anyhow!(
                "[Parse DevResponseCmd error] Payload not the String variant"
            ));
        }
    }

    pub fn handle_error_resp(&mut self, cmd: &str, error_id: usize) {
        trace!(
            "[LWNSIM][CMD_RESP][{:?}][ERROR]{:?}",
            cmd,
            CMD_ERROR_NAME[error_id]
        );

        if cmd == CMD_LINK_DEV {
            self.set_status(LwnsimStatus::ConnLinkDevNOK);
        } else if cmd == CMD_UNLINK_DEV {
            self.set_status(LwnsimStatus::ConnUnlinkDevNOK);
        } else if cmd == CMD_JOIN_REQUEST || cmd == CMD_SEND_UPLINK || cmd == CMD_RECV_DOWNLINK {
            self.push_lora_error_status(error_id);
        } else {
        }
    }

    pub fn handle_ok_resp(&mut self, cmd: &str, error_id: usize, object: &Value) {
        trace!(
            "[LWNSIM][CMD_RESP][{:?}]{:?}",
            cmd,
            serde_json::to_string(object)
        );

        if cmd == CMD_LINK_DEV {
        } else if cmd == CMD_UNLINK_DEV {
            self.set_status(LwnsimStatus::ConnUnlinkDevOK);
        } else if cmd == CMD_RECV_DOWNLINK {
            self.push_lora_error_status(error_id);
            if let Value::String(pl) = &object[0]["payload"] {
                LWNSIM.lock().unwrap().push_lora_recv_buffer(pl);
            } else {
                warn!("[LWNSIM] DevResponseCmd json error");
                return;
            }
        }
    }

    pub fn push_lora_error_status(&self, error_id: usize) {
        LORA.lock().unwrap().set_error_status(error_id);
    }
    pub fn push_lora_recv_buffer(&self, pl: &str) {
        LORA.lock().unwrap().set_recv_buf(pl);
    }
    pub fn push_lora_event(&self, event_val: usize) {
        LORA.lock().unwrap().handle_lora_event(event_val);
    }
}
