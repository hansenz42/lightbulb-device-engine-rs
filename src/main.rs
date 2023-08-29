mod http_server;
mod mqtt_client;
mod common;
use http_server::server::run as http_run;
use mqtt_client::client::run as mqtt_run;
use common::setting::Settings;
use common::logger::init_logger;
use std::{thread, time, error::Error};
use log::{info, warn};


fn main() -> Result<(), Box<dyn Error>> {
    // 设置 logger
    init_logger()?;

    let settings = Settings::get();
    info!("settings: {:?}", settings);
    Ok(())
}