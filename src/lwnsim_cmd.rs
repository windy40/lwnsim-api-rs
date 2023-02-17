#![allow(unused)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use typetag;

use log::error;
use std::error::Error;



use rust_socketio::Payload;
// simulator commands CMD_LINK_DEV,CMD_UNLINK_DEV,CMD_JOIN_REQUEST,CMD_SEND_UPLINK,CMD_RECV_DOWNLINK
pub static CMD_LINK_DEV: &str = "link-dev";
pub static CMD_UNLINK_DEV: &str = "unlink-dev";
pub static CMD_JOIN_REQUEST: &str = "join-request";
pub static CMD_SEND_UPLINK: &str = "send-uplink";
pub static CMD_RECV_DOWNLINK: &str = "recv-downlink";

// response-cmd errors
pub static DEV_CMD_OK: usize = 0;
pub static DEV_CMD_TIMEOUT: usize = 1;
pub static DEV_ERROR_NO_DEVICE_WITH_DEVEUI: usize = 2;
pub static DEV_ERROR_NIY: usize = 3;
pub static DEV_ERROR_DEVICE_NOT_LINKED: usize = 4;
pub static DEV_ERROR_DEVICE_TURNED_OFF: usize = 5;
pub static DEV_ERROR_DEVICE_NOT_JOINED: usize = 6;
pub static DEV_ERROR_DEVICE_ALREADY_JOINED: usize = 7;
pub static DEV_ERROR_NO_DATA_DOWNLINK_RECV: usize = 8;
pub static DEV_ERROR_SIMULATOR_NOT_RUNNING: usize = 9;
pub static CMD_ERROR_NAME: &'static [&'static str] = &[
    "DevCmdOK",
    "DevCmdTimeout",
    "NoDeviceWithDevEUI",
    "NIY",
    "DeviceNotlinked",
    "DeviceTurnedOff",
    "DeviceNotJoined",
    "DeviceAlreadyJoined",
    "NoDataDWrecv",
    "SimulatorNotRunning",
];

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
    fn get_error(&self) -> usize;    
    fn get_mtype(&self) -> &str;
    fn get_payload(&mut self) -> Option<String>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DevResponseCmd {
    pub cmd: String,
    pub error: usize,
}

#[typetag::serde]
impl DevResponseCmdTrait for DevResponseCmd {
    fn get_cmd(&self) -> &str {
        return &self.cmd;
    }
    fn get_error(&self) -> usize {
        return self.error;
    }

    fn get_mtype(&self) -> &str {
        return "";
    }

    fn get_payload(&mut self) -> Option<String>{
        return None;
    }
}

/* pub fn deser_payload(resp: Payload) -> Box<dyn DevResponseCmdTrait> {
let mut resp_struct : Box<dyn DevResponseCmdTrait> = Box::new(DevNoResponseCmd{}) ;

if let Payload::String(data) = resp {
    let object: Value = serde_json::from_str(&data).unwrap();

    if let Value::String(cmd_name) = &*object.get("cmd").unwrap() {
        if (cmd_name == CMD_LINK_DEV || cmd_name == CMD_UNLINK_DEV) {
                let tmp : DevResponseCmd = from_value(object).unwrap();
                resp_struct = Box::new(tmp);
        }
        else if cmd_name == CMD_RECV_DOWNLINK  {
                 let tmp: DevResponseRecvDownlinkCmd = from_value(object).unwrap();
                 resp_struct = Box::new(tmp);
        }
        else{
                let tmp = DevNoResponseCmd{};
                resp_struct = Box::new(tmp);
        }
    }else {
       error!("DevResponseCmd json error");
    }
}
else{
        error!("Payload : not the String variant error");
    }
    return resp_struct;
} */

#[derive(Debug, Serialize, Deserialize)]
pub struct DevNoResponseCmd {}

#[typetag::serde]
impl DevResponseCmdTrait for DevNoResponseCmd {
    fn get_cmd(&self) -> &str {
        return "";
    }
    fn get_error(&self) -> usize {
        return 0;
    }

    fn get_mtype(&self) -> &str {
        return "";
    }

    fn get_payload(&mut self) -> Option<String>{
        return None;
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DevResponseRecvDownlinkCmd {
    pub cmd: String,
    pub error: usize,
    pub mtype: String,
    pub payload: Option<String>,
}

#[typetag::serde]
impl DevResponseCmdTrait for DevResponseRecvDownlinkCmd {
    fn get_cmd(&self) -> &str {
        return &self.cmd;
    }
    fn get_error(&self) -> usize {
        return self.error;
    }

    fn get_mtype(&self) -> &str {
        return &self.mtype;
    }

    fn get_payload(&mut self) -> Option<String>{
        return self.payload.take();
    }
}
#[derive(Debug, Deserialize)]
pub struct DevLoRaEvent {
    event: usize,
}
