#![allow(unused)]

use serde::{Deserialize, Serialize};

// simulator commands
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
struct DevExecuteRecvDownlink {
    #[serde(rename = "Cmd")]
    cmd :       String,
    #[serde(rename = "Ack")]
    ack :       bool,   
    #[serde(rename = "DevEUI")]
    dev_eui :    String,
    #[serde(rename = "BufferSize")]
    buffer_size : i64,
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
struct DevExecuteSendUplink  {
    #[serde(rename = "Cmd")]
    cmd :    String,
    #[serde(rename = "DevEUI")]
    dev_eui : String,
    #[serde(rename = "Ack")] 
    ack :    bool, 
    #[serde(rename = "MType")]    
    mtype :  String,
    #[serde(rename = "Payload")]    
    payload : String,
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

pub trait DevResponseCmdTrait {
    fn get_cmd(&self) -> &str;
    fn get_error(&self) -> i64;
}

#[derive(Debug, Deserialize)]
pub struct DevResponseCmd {
    cmd: String,
    error: i64,
}

impl DevResponseCmdTrait for DevResponseCmd {
    fn get_cmd(&self) -> &str {
        return &self.cmd;
    }
    fn get_error(&self) -> i64 {
        return self.error;
    }
}

#[derive(Debug, Deserialize)]
struct DevNoResponseCmd {}

impl DevResponseCmdTrait for DevNoResponseCmd {
    fn get_cmd(&self) -> &str {
        return "";
    }
    fn get_error(&self) -> i64 {
        return 0;
    }
}

#[derive(Debug, Deserialize)]
struct DevResponseRecvDownlinkCmd  {
    cmd :    String,
    error :  i64, 
    mtype :  String,
    payload : String,
}

#[derive(Debug, Deserialize)]
struct DevLoRaEvent {
    event : i64,
}
