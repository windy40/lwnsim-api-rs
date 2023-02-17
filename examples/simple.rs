#![allow(unused)]
#![allow(while_true)]

use lwnsim_api_rs::lorawan::*;
use lwnsim_api_rs::lwnsim::*;
use lwnsim_api_rs::socket::*;

use std::{thread, time};
use std::sync::atomic::{AtomicBool, Ordering};
use ctrlc;

// log
use env_logger::{Builder, Env};
use log::{info, trace, warn, LevelFilter};

static URL: &str = "http://localhost:8000";
static DEV_EUI: &str = "359ac7cd01bc8aff";
static APP_KEY: &str = "f1c4081b61e9bee79bef58b5347e78a5";
static JOIN_EUI: &str = "0000000000000000";

static CTRLC_SEEN : AtomicBool = AtomicBool::new(false);

// log init
fn configure_log() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    Builder::from_env(env)
        .filter_module("lwnsim_api_rs", LevelFilter::Trace)
        .filter_module("simple", LevelFilter::Trace)
        .init();
}

/* pub fn log(msg: String) {
    if self.log_enable {
        let now: DateTime<Utc> = Utc::now();
        println!("{} [LWNSIM]{}", now.format("%Y-%m-%d %H:%M:%S"), msg);
    }
} */

fn main() {
    // The `Env` lets us tweak what the environment
    // variables to read are and what the default
    // value is if they're missing
    ctrlc::set_handler( || {
        info!("[EXAMPLE] received Ctrl+C!");
        CTRLC_SEEN.store(false,Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    configure_log();

    LWNSIM.lock().unwrap().set_dev_eui(DEV_EUI.to_string());

    let dur = time::Duration::from_secs(1);

    LWNSIM.lock().unwrap().connect(URL);

    thread::sleep(dur);

    LWNSIM.lock().unwrap().link_dev();

    let dev_eui=LORA.lock().unwrap().get_dev_eui().to_string();
    trace!("[EXAMPLE] linked to dev {:?}", dev_eui);

    thread::sleep(dur);

    //   lora=LoRa.LoRa( mode=LoRa.LORAWAN, region=LoRa.EU868, log_enable=True)

    //create an OTAA authentication parameters
    //app_eui = binascii.unhexlify('0000000000000000'.replace(' ',''))
    //app_key = binascii.unhexlify('2CC172969D5CC26382E0AD054568CE3E'.replace(' ',''))
    //app_key = binascii.unhexlify(''.replace(' ',''))

    trace!("[EXAMPLE] start join");
    LORA.lock().unwrap().join(
        OTAA,
        (JOIN_EUI.to_string(), APP_KEY.to_string()),
        Some(0),
        Some(0),
    );

    while !LORA.lock().unwrap().has_joined() {
        thread::sleep(dur);
        info!("Not yet joined...");
    }

    let mut s = Socket::new(AF_LORA, SOCK_RAW);
    s.setsockopt(SOL_LORA, SO_DR, 5);
    s.setsockopt(SOL_LORA, SO_CONFIRMED, 1);

    while true {
//        s.settimeout(Some(10));
        s.setblocking(true);

        let res = s.send("Hello");
        match res {
            Ok(()) => {
                info!(">>>>>> Hello >>>>>>>>");
            }
            Err(e) => {
                info!("error in sending {:?}", e);
            }
        };

        thread::sleep(dur);

        s.setblocking(false);
        s.settimeout(Some(3));
        let res = s.recv(2000);
        match res {
            Ok(resp) => {
                info!("<<<<<<<<<< {:?} <<<<<<<<<<<", resp);
            }
            Err(e) => {
                info!("error in receiving{:?}", e);
            }
        };

        if CTRLC_SEEN.load(Ordering::Relaxed) {  
            LWNSIM.lock().unwrap().disconnect();
        }

    }

    s.close();

    
}
