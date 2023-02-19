#![allow(unused)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use typetag;

use log::{error, warn};
use rust_socketio::Payload;
//use std::error::Error;
use super::error::{Error, Result};
use super::lora_events::LoraEvents;

// simulator commands CMD_LINK_DEV,CMD_UNLINK_DEV,CMD_JOIN_REQUEST,CMD_SEND_UPLINK,CMD_RECV_DOWNLINK
pub static CMD_LINK_DEV: &str = "link-dev";
pub static CMD_UNLINK_DEV: &str = "unlink-dev";
pub static CMD_JOIN_REQUEST: &str = "join-request";
pub static CMD_SEND_UPLINK: &str = "send-uplink";
pub static CMD_RECV_DOWNLINK: &str = "recv-downlink";

pub trait DevExecuteCmdTrait {
    fn get_cmd(&self) -> &str;
    fn get_ack(&self) -> bool;
    fn set_ack(&mut self, ack_cmd: bool);
    fn get_dev_eui(&self) -> &str;
}

#[derive(Debug, Serialize)]
pub struct DevExecuteCmd {
    #[serde(rename = "Cmd")]
    pub cmd: String,
    #[serde(rename = "Ack")]
    pub ack: bool,
    #[serde(rename = "DevEUI")]
    pub dev_eui: String,
}

impl DevExecuteCmdTrait for DevExecuteCmd {
    fn get_cmd(&self) -> &str {
        return &self.cmd;
    }
    fn get_ack(&self) -> bool {
        return self.ack;
    }
    fn set_ack(&mut self, ack_cmd: bool) -> () {
        return self.ack = ack_cmd;
    }
    fn get_dev_eui(&self) -> &str {
        return &self.dev_eui;
    }
}

#[derive(Debug, Serialize)]
pub struct DevExecuteRecvDownlink {
    #[serde(rename = "Cmd")]
    pub cmd: String,
    #[serde(rename = "Ack")]
    pub ack: bool,
    #[serde(rename = "DevEUI")]
    pub dev_eui: String,
    #[serde(rename = "BufferSize")]
    pub buffer_size: usize,
}
impl DevExecuteCmdTrait for DevExecuteRecvDownlink {
    fn get_cmd(&self) -> &str {
        return &self.cmd;
    }
    fn get_ack(&self) -> bool {
        return self.ack;
    }
    fn set_ack(&mut self, ack_cmd: bool) -> () {
        return self.ack = ack_cmd;
    }
    fn get_dev_eui(&self) -> &str {
        return &self.dev_eui;
    }
}
#[derive(Debug, Serialize)]
pub struct DevExecuteSendUplink {
    #[serde(rename = "Cmd")]
    pub cmd: String,
    #[serde(rename = "DevEUI")]
    pub dev_eui: String,
    #[serde(rename = "Ack")]
    pub ack: bool,
    #[serde(rename = "MType")]
    pub mtype: String,
    #[serde(rename = "Payload")]
    pub payload: String,
}

impl DevExecuteCmdTrait for DevExecuteSendUplink {
    fn get_cmd(&self) -> &str {
        return &self.cmd;
    }
    fn get_ack(&self) -> bool {
        return self.ack;
    }
    fn set_ack(&mut self, ack_cmd: bool) -> () {
        return self.ack = ack_cmd;
    }
    fn get_dev_eui(&self) -> &str {
        return &self.dev_eui;
    }
}

#[derive(Debug, Deserialize)]
pub struct DevAckCmd {
    #[serde(rename = "Cmd")]
    cmd: String,
    #[serde(rename = "Args")]
    args: String,
}

#[typetag::serde(tag = "type")]
pub trait DevResponseCmdTrait {
    fn get_cmd(&self) -> &str;
    fn get_error(&self) -> CmdErrorKind;
    fn get_mtype(&self) -> &str {
        return "";
    }
    fn get_payload(&mut self) -> String {
        return "".to_string();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DevResponseCmd {
    pub cmd: String,
    pub error: CmdErrorKind,
}

#[typetag::serde]
impl DevResponseCmdTrait for DevResponseCmd {
    fn get_cmd(&self) -> &str {
        return &self.cmd;
    }
    fn get_error(&self) -> CmdErrorKind {
        return self.error.clone();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DevNoResponseCmd {}

#[typetag::serde]
impl DevResponseCmdTrait for DevNoResponseCmd {
    fn get_cmd(&self) -> &str {
        return "";
    }
    fn get_error(&self) -> CmdErrorKind {
        return CmdErrorKind::DevCmdOK;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DevResponseRecvDownlinkCmd {
    pub cmd: String,
    pub error: CmdErrorKind,
    pub mtype: String,
    pub payload: Option<String>, // is an Option so that String can be moved out with take()
}

#[typetag::serde]
impl DevResponseCmdTrait for DevResponseRecvDownlinkCmd {
    fn get_cmd(&self) -> &str {
        return &self.cmd;
    }
    fn get_error(&self) -> CmdErrorKind {
        return self.error.clone();
    }
    fn get_mtype(&self) -> &str {
        return &self.mtype;
    }
    fn get_payload(&mut self) -> String {
        return self.payload.take().unwrap();
    }
}

pub fn parse_resp_cmd(resp_msg: Payload) -> Result<Box<dyn DevResponseCmdTrait>> {
    if let Payload::String(json_str) = resp_msg {
        let object: Value = serde_json::from_str(&json_str).unwrap();
        if let Value::String(cmd_name) = &object[0]["cmd"] {
            if cmd_name == CMD_LINK_DEV || cmd_name == CMD_UNLINK_DEV {
                let resp_cmd: DevResponseCmd =
                    serde_json::from_value(object[0].clone()).expect("json deserialization failed");
                return Ok(Box::new(resp_cmd));
            } else
            /*if cmd_name == CMD_RECV_DOWNLINK*/
            {
                let resp_cmd: DevResponseRecvDownlinkCmd =
                    serde_json::from_value(object[0].clone()).expect("json deserialization failed");
                return Ok(Box::new(resp_cmd));
            }
        } else {
            warn!("[LWNSIM] DevResponseCmd json error");
            return Err(Error::CmdError(CmdErrorKind::PayloadJsonError));
        }
    } else {
        return Err(Error::CmdError(CmdErrorKind::PayloadNotStringVariant));
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DevLoraEvent {
    pub event: LoraEvents,
}



use serde_repr::*;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u32)]
pub enum CmdErrorKind {
    DevCmdOK=0,
    DevCmdTimeout=1,
    NoDeviceWithDevEUI=2,
    NIY=3,
    DeviceNotlinked=4,
    DeviceTurnedOff=5,
    DeviceNotJoined=6,
    DeviceAlreadyJoined=7,
    NoDataDWrecv=8,
    SimulatorNotRunning=9,
    PayloadNotStringVariant=10,
    PayloadJsonError=11,
    UnexpectedError=12,
}
use std::fmt;
impl fmt::Display for CmdErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            CmdErrorKind::DevCmdOK => "Cmd OK",
            CmdErrorKind::DevCmdTimeout => "Cmd timeout",
            CmdErrorKind::NoDeviceWithDevEUI => "No device found with given devEUI",
            CmdErrorKind::NIY => "Not implemented yet",
            CmdErrorKind::DeviceNotlinked => "Device not linked",
            CmdErrorKind::DeviceTurnedOff => "Device turned off or simulator stopped",
            CmdErrorKind::DeviceNotJoined => "Device not joined",
            CmdErrorKind::DeviceAlreadyJoined => {
                "Device already joined (rejoining not implemented yet)"
            }
            CmdErrorKind::NoDataDWrecv => "No downlink userdata received ",
            CmdErrorKind::SimulatorNotRunning => "Simulator not running",
            CmdErrorKind::PayloadNotStringVariant => "Payload not the String variant",
            CmdErrorKind::PayloadJsonError => "Error in parsing json",
            CmdErrorKind::UnexpectedError => "Unexpected error",
        };
        write!(f, "{}", name)
    }
}


