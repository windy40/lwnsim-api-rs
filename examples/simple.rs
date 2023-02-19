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
use log::{error, info, trace, warn, LevelFilter};

static URL: &str = "http://localhost:8000";
static DEV_EUI: &str = "359ac7cd01bc8aff";
static APP_KEY: &str = "f1c4081b61e9bee79bef58b5347e78a5"; // set as device info in LWNSim
static JOIN_EUI: &str = "0000000000000000"; // set as device info in LWNSim

static CTRLC_SEEN : AtomicBool = AtomicBool::new(false);

// log init
fn configure_log() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    Builder::from_env(env)
        .filter_module("lwnsim_api_rs", LevelFilter::Trace)
        .filter_module("simple", LevelFilter::Info)
        .init();
}


fn main() {
    // The `Env` lets us tweak what the environment
    // variables to read are and what the default
    // value is if they're missing
    ctrlc::set_handler( || {
        info!("[EXAMPLE] received Ctrl+C!");
        CTRLC_SEEN.store(true,Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    let dur_1s = time::Duration::from_secs(1);

    configure_log();
// creates lazily LWNSIM and connects to LWN simulator
    LWNSIM.lock().unwrap().connect(URL, DEV_EUI);

    thread::sleep(dur_1s);

    info!("[EXAMPLE] link to dev {:?}", DEV_EUI);
    LWNSIM.lock().unwrap().link_dev();


    thread::sleep(dur_1s);

    //   lora=LoRa.LoRa( mode=LoRa.LORAWAN, region=LoRa.EU868, log_enable=True)

    //create an OTAA authentication parameters
    //app_eui = binascii.unhexlify('0000000000000000'.replace(' ',''))
    //app_key = binascii.unhexlify('2CC172969D5CC26382E0AD054568CE3E'.replace(' ',''))
    //app_key = binascii.unhexlify(''.replace(' ',''))

    info!("[EXAMPLE] start dev joining");
    LORA.lock().unwrap().join(
        OTAA,
        (JOIN_EUI.to_string(), APP_KEY.to_string()),
        Some(0),  // not used 
        Some(0), // not used LWNSim manages DR depending on device info
    );

    while !LORA.lock().unwrap().has_joined() {
        thread::sleep(dur_1s);
        info!("[EXAMPLE] Not yet joined...");
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
                info!("[EXAMPLE]>>>>>> Hello >>>>>>>>");
            }
            Err(e) => {
                error!("[EXAMPLE]send error : {:?}", e);
                break;
            }
        };

        thread::sleep(dur_1s);

        s.setblocking(false);
        s.settimeout(Some(3));
        let res = s.recv(2000);
        match res {
            Ok(resp) => {
                info!("[EXAMPLE]<<<<<<<<<< {:?} <<<<<<<<<<<", resp);
            }
            Err(e) => {
                error!("[EXAMPLE] receive error : {:?}", e);
                break;
            }
        };

        if CTRLC_SEEN.load(Ordering::Relaxed) {  
            LWNSIM.lock().unwrap().disconnect();
        }

    }

    LWNSIM.lock().unwrap().disconnect();

    
}
