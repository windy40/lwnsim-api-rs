#![allow(unused)]
use lwnsim_api_rs::lwnsim::Lwnsim;
use std::{thread, time};

// log
use env_logger::{Builder, Env};
use log::{info, trace, warn, LevelFilter};

static URL: &str = "http://localhost:8000";
static DEV_EUI: &str = "359ac7cd01bc8aff";
// join_eui = app_eui = binascii.unhexlify('0000000000000000'.replace(' ',''))
//  app_key = binascii.unhexlify('f1c4081b61e9bee79bef58b5347e78a5'.replace(' ',''))

// log init
fn configure_log() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    Builder::from_env(env)
        .filter_module("lwnsim_api_rs", LevelFilter::Trace)
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

    configure_log();

    let lwnsim: &mut Lwnsim = &mut Lwnsim::new(DEV_EUI.to_string());
    let dur = time::Duration::from_secs(3);

    lwnsim.connect(URL);

    thread::sleep(dur);

    lwnsim.link_dev();

    thread::sleep(dur);

    lwnsim.disconnect();
}
