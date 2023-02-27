#![allow(unused)]

use super::lora_dev::*;
use super::lora_events::{LORA_EVENTS,LoraEvents};
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

        if self.blocking {
            debug!("[SOCKET][blocking send (timeout= {:?})]MType= {} data= {}",self.timeout, mtype, data);           
            LORA_EVENTS.lock().unwrap().clear_events(LoraEvents::TX_PACKET_EVENT|LoraEvents::TX_FAILED_EVENT);
        }else{
            debug!("[SOCKET][send]MType= {} data= {}", mtype, data);
        }

        LORA.lock().unwrap().send(mtype, data)?;

        if self.blocking {
            if let Some(dur) = self.timeout {
                let start_time = Instant::now();
                while ! LORA_EVENTS.lock().unwrap().intersects(LoraEvents::TX_PACKET_EVENT|LoraEvents::TX_FAILED_EVENT) {
                    if start_time.elapsed().as_secs() > dur.try_into().unwrap() {
                        return Err(Error::CmdError(CmdErrorKind::DevCmdTimeout));
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            } else {
                while ! LORA_EVENTS.lock().unwrap().intersects(LoraEvents::TX_PACKET_EVENT|LoraEvents::TX_FAILED_EVENT) {
                    thread::sleep(Duration::from_secs(1));
                }
            }
            if LORA_EVENTS.lock().unwrap().contains_and_remove_event(LoraEvents::TX_PACKET_EVENT) {
                debug!("[SOCKET][blocking send]OK");
            }
            if LORA_EVENTS.lock().unwrap().contains_and_remove_event(LoraEvents::TX_FAILED_EVENT){
                debug!("[SOCKET][blocking send]failed");
            }
        }
        Ok(())
    }

    pub fn recv(&self, buffersize: usize) -> Result<String> {
        if self.blocking {
            debug!("[SOCKET][blocking recv]Buffersize={}", buffersize);
            LORA_EVENTS.lock().unwrap().clear_events(LoraEvents::RX_PACKET_EVENT);
        }else {
            debug!("[SOCKET][recv]Buffersize={}", buffersize);
        }
        let mut recv_buf = LORA.lock().unwrap().recv(buffersize);
        match recv_buf {
            Err(Error::CmdError(CmdErrorKind::NoDataDWrecv)) => {
                if self.blocking {
                    if let Some(dur) = self.timeout {

                        let start_time = Instant::now();
                        while ! LORA_EVENTS.lock().unwrap().contains(LoraEvents::RX_PACKET_EVENT) {
                            if start_time.elapsed().as_secs() > dur.try_into().unwrap() {
                                debug!("[SOCKET][blocking recv][error]timeout");
                                return Err(Error::CmdError(CmdErrorKind::DevCmdTimeout));
                            }
                            thread::sleep(Duration::from_secs(1));
                        }

                        LORA_EVENTS.lock().unwrap().clear_events(LoraEvents::RX_PACKET_EVENT);
                        return LORA.lock().unwrap().recv(buffersize);

                    } else { // due to Lorawan protocol, blocking without timeout will prevent sending new Lora frames and consequently receiving data
                        while ! LORA_EVENTS.lock().unwrap().contains(LoraEvents::RX_PACKET_EVENT) {
                            thread::sleep(Duration::from_secs(1));
                        }
                        LORA_EVENTS.lock().unwrap().clear_events(LoraEvents::RX_PACKET_EVENT);
                        return LORA.lock().unwrap().recv(buffersize);
                    }
                    debug!("[SOCKET][blocking recv] success");
                }else{
                    debug!("[SOCKET][recv][error]no downlink data received");
                    return Err(Error::CmdError(CmdErrorKind::NoDataDWrecv))
                }
            }
            Err(e)=> return Err(e),
            Ok(s) => return Ok(s),        
        };

    }
}
