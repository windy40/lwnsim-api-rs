#![allow(unused)]

use super::lorawan::*;
// log
use log::{debug, info, trace, warn};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};

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
    if events & TX_PACKET_EVENT {
        socket.tx_packet_event=true;
    }
    if events & TX_FAILED_EVENT{
        socket.tx_failed_event=true;
    }
    return;
} */

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

        // self.stack.callback(trigger=(LoRa.TX_PACKET_EVENT | LoRa.TX_FAILED_EVENT), handler=self.set_blocking_send_status, arg=())
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

    pub fn send(&self, data: &str) -> Result<()> {
        let mut mtype: &str = UNCONFIRMED_DATA_UP;
        if self.confirmed {
            mtype = CONFIRMED_DATA_UP;
        }
        trace!("[SOCKET][send]MType= {} data= {}", mtype, data);
        LORA.lock().unwrap().send(mtype, data);
        if self.blocking {
            let mut events = LORA.lock().unwrap().events();
            if let Some(dur) = self.timeout {
                let start_time = Instant::now();
                while events & TX_PACKET_EVENT == 0 && events & TX_FAILED_EVENT == 0 {
                    if start_time.elapsed().as_secs() > dur.try_into().unwrap() {
                        return Err(anyhow!("timeout"));
                    }
                    thread::sleep(Duration::from_secs(1));
                    events = LORA.lock().unwrap().events();
                }
            } else {
                while events & TX_PACKET_EVENT == 0 && events & TX_FAILED_EVENT == 0 {
                    thread::sleep(Duration::from_secs(1));
                    events = LORA.lock().unwrap().events();
                }
            }
            if events & TX_PACKET_EVENT != 0 {
                trace!("[blocking send] sucess");
            }
            if events & TX_FAILED_EVENT != 0 {
                trace!("[blocking send] failed");
            }
        }
        Ok(())
    }

    pub fn recv(&self, buffersize: usize) -> Result<String> {
        trace!("[SOCKET][recv]Buffersize={}", buffersize);
        let mut recv_buf = LORA.lock().unwrap().recv(buffersize);
        match recv_buf {
            None => {
                if self.blocking {
                    if let Some(dur) = self.timeout {
                        let start_time = Instant::now();

                        while LORA.lock().unwrap().events() & RX_PACKET_EVENT == 0 {
                            if start_time.elapsed().as_secs() > dur.try_into().unwrap() {
                                return Err(anyhow!("timeout"));
                            }
                            thread::sleep(Duration::from_secs(1));
                        }
                        recv_buf = LORA.lock().unwrap().recv(buffersize);
                    } else {
                        while LORA.lock().unwrap().events() & RX_PACKET_EVENT == 0 {
                            thread::sleep(Duration::from_secs(1));
                        }
                        recv_buf = LORA.lock().unwrap().recv(buffersize);
                    }
                    trace!("[blocking recv] sucess");
                }

            }
            _ => {}
        }
        if let Some(buf) =recv_buf {
            return Ok(buf);
        }else{
            return Err(anyhow!("something went wrong in recv"));
        }
    }
}
