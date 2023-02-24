use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload, RawClient};

use std::time::Duration;

use lazy_static::lazy_static;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;



use super::error::{Error, Result};
use super::lora_events::LoraEvents;
use super::lora_dev::LORA;
use super::lwnsim_cmd::*;

// log
use log::{info, trace, warn};

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

// events sent by simulator, call back function defined in ClientBuilder.on()
static DEV_EVENT_ACK_CMD: &str = "ack-cmd";
static DEV_EVENT_LORA: &str = "lora-event";
//static DEV_EVENT_LOG: &str = "dev-log"; // unused
// static DEV_EVENT_ERROR: &str = "dev-error"; //unused
// static DEV_EVENT_RESPONSE_CMD: &str = "response-cmd"; // is handled by emit_with_ack call back function

// use CMD_<cmd name> defined in lwnsim_cmd.rs as event name
// static DEV_EVENT_LINK_DEV: &str = "link-dev";
// static DEV_EVENT_UNLINK_DEV: &str = "unlink-dev";
// static DEV_EVENT_JOIN_REQUEST: &str = "join-request";
// static DEV_EVENT_SEND_UPLINK: &str = "send-uplink";
// static DEV_EVENT_RECV_DOWNLINK: &str = "recv-downlink";

lazy_static! {
    pub static ref LWNSIM: Mutex<Lwnsim> = Mutex::new(Lwnsim::new());
}

pub enum SendMode {
    Call,
    Emit,
}

pub struct Lwnsim {
    url: Option<String>,
    dev_eui: Option<String>,
    socket: Option<Client>,
    status: LwnsimStatus,
    ack_cmd: bool,
    timeout_cmd: u64,

    //    handle_response_cmd: Box<dyn FnMut(Payload,RawClient)->()>,
}

impl Lwnsim {
    pub fn new() -> Lwnsim {
        Lwnsim {
            status: LwnsimStatus::ConnNOK,
            socket: None,
            ack_cmd: true,
            timeout_cmd: 10,
            dev_eui: None,
            url: None,
        }
    }

    pub fn get_dev_eui(&self) -> &str {
        return self.dev_eui.as_ref().expect("[LWNSIM] devEUI not defined");
    }

/*     fn set_cmd_timeout(&mut self, timeout: u64){
        self.timeout_cmd=timeout;
    } */


    pub fn connect(&mut self, url: &str, dev_eui: &str) {
        self.url = Some(url.to_string());
        self.dev_eui= Some(dev_eui.to_string());
        self.status = LwnsimStatus::ConnInit;

        let socket = ClientBuilder::new(url.to_string())
            .namespace("/dev")
            .on("open", |_, _| info!("[LWNSIM][Socket event] Connected"))
            .on("close", |_, _| info!("[LWNSIM][Socket event] Disconnected"))
            .on(DEV_EVENT_ACK_CMD, |payload, _: RawClient| {
                if let Payload::String(s) = payload {
                    trace!("[LWNSIM][CMD_ACK][cmd]{:?}", s);
                };
            })
            .on(DEV_EVENT_LORA, |payload, _: RawClient| {

                if let Payload::String(pl_str) = payload {
                    trace!("[LWNSIM][LORA EVENT]{:?}", pl_str);
                    let lora_event: DevLoraEvent = serde_json::from_str(&pl_str)
                        .expect("[LWNSIM][ParseDevLoraEventError] json error");
                    LWNSIM.lock().unwrap().push_lora_event(lora_event.event);
                } else {
                    warn!("[LWNSIM][ParseDevLoraEventError] not the String variant");
                }
            })
            // .on("error", |err, _| eprintln!("Error: {:#?}", err))
            .opening_header("accept-encoding", "application/json")
            .connect()
            .expect("Connection failed");
        trace!("[LWNSIM][connect]");
        self.socket = Some(socket);
        self.status = LwnsimStatus::ConnOK;
    }

    pub fn disconnect(&self) {
        if let Some(s) = &self.socket {
            trace!("[LWNSIM][disconnect]");
            s.disconnect().expect("Disconnect failed");
        }
    }



    pub fn send_cmd(
        &mut self,
        mut msg: impl DevExecuteCmdTrait + serde::Serialize,
        mode: SendMode,
    ) -> Result<Option<Box<dyn ResponseCmdTrait>>> {

        if self.ack_cmd {
            msg.set_ack(true);
        }
        let msg_json = serde_json::to_value(&msg).expect("serialization to value failed");
        let event_name=msg.get_cmd();
        match mode {
            SendMode::Emit => {
                trace!(
                    "[LWNSIM][CMD_EMIT][{}]{:?}",
                    event_name,
                    //serde_json::to_string(&msg)
                    msg_json
                );
                self.socket
                    .as_ref()
                    .expect("socket unset")
                    .emit(event_name, msg_json)
                    .expect("emit failed");
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
                        event_name,
                        msg_json,
                        Duration::from_secs(2),
                        move |message: Payload, _: RawClient| {
                            trace!("[LWNSIM][CMD_RESP]{:?}", message);
                            // send the result to the channel
                            tx.send(message).unwrap();
                        },
                    ).expect("emit_with_ack failed");

                match rx.recv_timeout(Duration::from_secs(self.timeout_cmd)){
                    Ok(resp_msg)=> {
                        let resp_cmd= parse_resp_cmd(resp_msg)?;
                        // handle simulator level errors 
                        if resp_cmd.get_error() == CmdErrorKind::SimulatorNotRunning
                            || resp_cmd.get_error() == CmdErrorKind::NoDeviceWithDevEUI
                        {
                            //self.set_status(LwnsimStatus::ConnUnlinkDevOK); //automatically unlink device 
                            self.disconnect(); 
                            return Err(Error::CmdError(resp_cmd.get_error()));
                        } else {
                            // return command response whatever the cmd error status 
                            return Ok(Some(resp_cmd));
                        }
                    },
                    Err(e) => {
                        trace!("[LWNSIM][CMD_RESP][TIMEOUT]{:?}", e);
                        return Err(Error::CmdError(CmdErrorKind::DevCmdTimeout));},
                }
            }
        }
    }


    pub fn push_lora_event(&self, event_val: LoraEvents) {
        LORA.lock().unwrap().handle_lora_event(event_val);
    }
}
