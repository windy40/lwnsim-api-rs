use bitflags::bitflags;
//use serde::{Serialize, Deserialize};
use serde_derive::*;
use std::fmt;

use lazy_static::lazy_static;
use std::sync::Mutex;

use super::lora_dev::{LoraDevStatus, LORA};
use log::trace;

bitflags! {
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct LoraEvents: u32 {
    const  RX_PACKET_EVENT = 1;
    const  TX_PACKET_EVENT = 2;
    const  TX_FAILED_EVENT = 4;
    const JOIN_ACCEPT_EVENT = 16;
    const  UNJOIN_EVENT = 32;
    }
}

lazy_static! {
    pub static ref LORA_EVENTS: Mutex<LoraEvents> = Mutex::new(LoraEvents::new());
}

impl LoraEvents {
    pub fn new() -> LoraEvents {
        LoraEvents::empty()
    }

    pub fn handle_lora_event(&mut self, event_val: LoraEvents) {
        if LoraEvents::JOIN_ACCEPT_EVENT == event_val {
            self.insert(LoraEvents::JOIN_ACCEPT_EVENT);
            trace!("[LORA_EVENTS]{:?}", self);
            LORA.lock().unwrap().set_status(LoraDevStatus::Joined);
            return;
        } else if event_val == LoraEvents::UNJOIN_EVENT {
            self.insert(LoraEvents::UNJOIN_EVENT);
            trace!("[LORA_EVENTS]{:?}", self);
            LORA.lock().unwrap().set_status(LoraDevStatus::Unjoined);
            return;
        }
        // set TX and RX type events
        // these events are used for blocking send and receive and possibly for async version in the futures
        self.insert(event_val);
        trace!("[LORA_EVENTS]{:?}", self);
    }

    pub fn clear_events(&mut self, evts: LoraEvents) {
        self.remove(evts);
        trace!("[LORA_EVENTS]{:?}", self);
    }

    pub fn contains_and_remove_event(&mut self, lora_event: LoraEvents) -> bool {
        if self.contains(lora_event) {
            self.remove(lora_event);
            return true;
        } else {
            return false;
        }
    }
}

impl fmt::Display for LoraEvents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:032b}", self.bits)
    }
}
