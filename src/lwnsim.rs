#![allow(unused)]

use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde::{Deserialize, Serialize};
use serde_json::{json, value};
use std::error::Error;

use chrono::{DateTime, Utc};
use std::time::Duration;

use super::lwnsim_cmd::*;
use super::lwnsim_cmd::{CMD_LINK_DEV, CMD_UNLINK_DEV,CMD_JOIN_REQUEST,CMD_SEND_UPLINK,CMD_RECV_DOWNLINK};

// log
use log::{info, trace, warn};

enum LwnsimStatus {
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

pub static LWNSIM: Option<&Lwnsim> = None;

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
    pub fn new(dev_eui: String) -> Lwnsim {
        Lwnsim {
            status: LwnsimStatus::ConnNOK,
            socket: None,
            ack_cmd: true,
            dev_eui: dev_eui,
        }
    }

    pub fn connect(&mut self, url: &str) {
        self.status = LwnsimStatus::ConnInit;

        let socket = ClientBuilder::new(url.to_string())
            .namespace("/dev")
            .on("open", |_, _| info!("Connected"))
            .on("close", |_, _| info!("Disconnected !"))
            .on(DEV_EVENT_ACK_CMD, |payload, _: RawClient| {
                if let Payload::String(s) = payload {
                    trace!("[CMD_ACK][cmd]{:?}", s);
                };
            })
            .on(DEV_EVENT_LORA, |payload, _: RawClient| {
                if let Payload::String(s) = payload {
                    trace!("[LORA][event]{:?}", s);
                };
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
        self.send_cmd(msg, SendMode::Call);
    }

    pub fn unlink_dev(&mut self) {
        let msg = DevExecuteCmd {
            cmd: CMD_UNLINK_DEV.to_string(),
            ack: true,
            dev_eui: (&self.dev_eui).to_string(),
        };
        self.status = LwnsimStatus::ConnUnlinkDevInit;
        self.send_cmd(msg, SendMode::Call);
    }

    pub fn send_cmd(&self, mut msg: impl DevExecuteCmdTrait + serde::Serialize, mode: SendMode) {
        if self.ack_cmd {
            msg.set_ack(true);
        }
        let msg_json = serde_json::to_value(&msg).expect("serialization to value failed");

        match mode {
            SendMode::Emit => {
                trace!(
                    "[CMD_EMIT][{}]{:?}",
                    msg.get_cmd(),
                    serde_json::to_string(&msg)
                );
                self.socket
                    .as_ref()
                    .expect("socket unset")
                    .emit(msg.get_cmd(), msg_json)
                    .expect("emit failed");
            }
            SendMode::Call => {
                trace!(
                    "[CMD_CALL][{}]{:?}",
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
                        |message: Payload, _: RawClient| {
                            trace!("[CMD_RESP]{:?}", message);
                        },
                    )
                    .expect("emit_with_ack failed")
            }
        }
    }
}
