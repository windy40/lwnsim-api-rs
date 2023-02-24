#![allow(unused)]

use super::lora_dev::*;
use super::lora_events::LoraEvents;
use super::lwnsim_cmd::CmdErrorKind;
use super::error::{Result,Error};
// log
use log::{debug, info, trace, warn};
use std::thread;
use std::time::{Duration, Instant};

pub static AF_LORA: usize = 1;
pub static SOCK_RAW: usize = 1;

pub static SOL_LORA: usize = 1;
// socket options
pub static SO_DR: usize = 1;
pub static SO_CONFIRMED: usize = 2;

pub static UNCONFIRMED_DATA_UP: &str = "UnconfirmedDataUp";
pub static CONFIRMED_DATA_UP: &str = "ConfirmedDataUp";

/* fn set_blocking_send_status(){
    let events = LORAWAN.events();
    if events & LoraEvents::TX_PACKET_EVENT {
        socket.tx_packet_event=true;
    }
    if events & LoraEvents::TX_FAILED_EVENT{
        socket.tx_failed_event=true;
    }
    return;
} */

#[derive(Debug)]
pub struct Socket {
    confirmed: bool,
    blocking: bool,
    timeout: Option<usize>,
    dr: usize,
}


impl Socket {
    pub fn new(af: usize, socket_t: usize) -> Socket {
        assert_eq!(af, AF_LORA);
        assert_eq!(socket_t, SOCK_RAW);
        Socket {
            confirmed: false,
            blocking: false,
            timeout: None,
            dr: 0,
        }

        // self.stack.callback(trigger=(LoRa.LoraEvents::TX_PACKET_EVENT | LoRa.LoraEvents::TX_FAILED_EVENT), handler=self.set_blocking_send_status, arg=())
    }

    pub fn setsockopt(&mut self, level: usize, optname: usize, value: usize) {
        assert_eq!(level, SOL_LORA);

        if optname == SO_CONFIRMED {
            self.confirmed = true;
        } else if value == SO_DR {
            self.dr = value;
        }
    }

    pub fn setblocking(&mut self, block: bool) {
        self.blocking = block;
    }

    pub fn settimeout(&mut self, to: Option<usize>) {
        self.timeout = to;
    }

    pub fn close(&self) {}

/// send a string as a Lora data payload
/// the payload will be encoded as base64 by the simulator
    pub fn send(&self, data: &str) -> Result<()> {
        let mut mtype: &str = UNCONFIRMED_DATA_UP;
        if self.confirmed {
            mtype = CONFIRMED_DATA_UP;
        }
        trace!("[SOCKET][send]MType= {} data= {}", mtype, data);
        LORA.lock().unwrap().send(mtype, data)?;
        if self.blocking {
            let mut events = LORA.lock().unwrap().events();
            if let Some(dur) = self.timeout {
                let start_time = Instant::now();
                while events & LoraEvents::TX_PACKET_EVENT == LoraEvents::NO_LORA_EVENT && events & LoraEvents::TX_FAILED_EVENT == LoraEvents::NO_LORA_EVENT {
                    if start_time.elapsed().as_secs() > dur.try_into().unwrap() {
                        return Err(Error::CmdError(CmdErrorKind::DevCmdTimeout));
                    }
                    thread::sleep(Duration::from_secs(1));
                    events = LORA.lock().unwrap().events();
                }
            } else {
                while events & LoraEvents::TX_PACKET_EVENT == LoraEvents::NO_LORA_EVENT && events & LoraEvents::TX_FAILED_EVENT == LoraEvents::NO_LORA_EVENT {
                    thread::sleep(Duration::from_secs(1));
                    events = LORA.lock().unwrap().events();
                }
            }
            if events & LoraEvents::TX_PACKET_EVENT != LoraEvents::NO_LORA_EVENT {
                trace!("[SOCKET][blocking send] sucess");
            }
            if events & LoraEvents::TX_FAILED_EVENT != LoraEvents::NO_LORA_EVENT {
                trace!("[SOCKET][blocking send] failed");
            }
        }
        Ok(())
    }

    pub fn recv(&self, buffersize: usize) -> Result<String> {
        trace!("[SOCKET][recv]Buffersize={}", buffersize);
        let mut recv_buf = LORA.lock().unwrap().recv(buffersize);
        match recv_buf {
            Err(Error::CmdError(CmdErrorKind::NoDataDWrecv)) => {
                if self.blocking {
                    if let Some(dur) = self.timeout {

                        let start_time = Instant::now();
                        while LORA.lock().unwrap().events() & LoraEvents::RX_PACKET_EVENT == LoraEvents::NO_LORA_EVENT {
                            if start_time.elapsed().as_secs() > dur.try_into().unwrap() {
                                return Err(Error::CmdError(CmdErrorKind::DevCmdTimeout));
                            }
                            thread::sleep(Duration::from_secs(1));
                        }

                        return LORA.lock().unwrap().recv(buffersize);
                    } else {
                        while LORA.lock().unwrap().events() & LoraEvents::RX_PACKET_EVENT == LoraEvents::NO_LORA_EVENT {
                            thread::sleep(Duration::from_secs(1));
                        }

                        return LORA.lock().unwrap().recv(buffersize);
                    }
                    trace!("[SOCKET][blocking recv] sucess");
                }else{
                    return Err(Error::CmdError(CmdErrorKind::NoDataDWrecv))
                }
            }
            Err(e)=> return Err(e),
            Ok(s) => return Ok(s),        
        };

    }
}
