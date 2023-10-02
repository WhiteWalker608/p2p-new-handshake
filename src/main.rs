#[macro_use]
extern crate lazy_static;
use std::env;
use std::error::Error;

mod config;
mod handshake;
mod types;
// global config
lazy_static! {
    static ref CONFIG: config::Configuration =
        config::Configuration::new().expect("config can be loaded");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let address = &args[1];
    let mut btc = handshake::btc::Btc::new(CONFIG.btc.buffersize as usize);
    let success = handshake::handshake(
        &mut btc,
        CONFIG.btc.tcptimeout,
        CONFIG.btc.handshaketimeout,
        address,
        &CONFIG.btc.useragent,
    )
    .await?;
    if !success {
        println!("handshake failed");
    }
    Ok(())
}
