mod http_server;
mod mqtt_client;
mod common;
mod driver;
mod device_controller;
mod file_controller;
mod entity;
mod util;
use ctrlc;
use http_server::server::CustomHttpServer;
use common::setting::Settings;
use common::logger::init_logger;
use common::sqlite::SqliteConnection;
use std::error::Error;
// use crate::device_controller::device_manager::DeviceManager;
use crate::file_controller::file_controller::FileController;
use log;
use dotenv::dotenv;
use tokio;

// #[macro_use] extern crate log;

const LOG_TAG: &str = "main";

fn main() {

}
