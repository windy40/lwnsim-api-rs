use bitflags::bitflags;
//use serde::{Serialize, Deserialize};
use serde_derive::*;
use std::fmt;

bitflags! {
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct LoraEvents: u32 {
       const NO_LORA_EVENT = 0;
const  RX_PACKET_EVENT = 1;
const  TX_PACKET_EVENT = 2;
 const  TX_FAILED_EVENT = 4;
 const JOIN_ACCEPT_EVENT = 16;
 const  UNJOIN_EVENT = 32;
    }
}

impl LoraEvents {
    pub fn clear(&mut self) -> &mut LoraEvents {
        self.bits = 0;
        self
    }
}

impl fmt::Display for LoraEvents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:032b}", self.bits)
    }
}
