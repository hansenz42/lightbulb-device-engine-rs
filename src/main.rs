use std::{sync::mpsc, thread};

use common::logger::init_logger;
use device_controller::device_controller::DeviceController;
use mqtt_client::client::MqttClient;

mod http_server;
mod mqtt_client;
mod common;
mod driver;
mod device_controller;
mod file_controller;
mod entity;
mod util;

// #[macro_use] extern crate log;

const LOG_TAG: &str = "main";
const SLEEP_INTERVAL: u64 = 1000;

fn main() {
    init_logger().expect("Fail to initialize logger");

    let (device_to_mqtt_tx, device_to_mqtt_rx) = mpsc::channel();
    let (mqtt_to_device_tx, mqtt_to_device_rx) = mpsc::channel();

    let device_controller = DeviceController::new();
    let mqtt_client = MqttClient::new();

    let mut handle_vec = device_controller.start(device_to_mqtt_tx, mqtt_to_device_rx).expect("Failed to start device controller");
    let handle = mqtt_client.start(mqtt_to_device_tx, device_to_mqtt_rx);
    handle_vec.push(handle);

    info!(LOG_TAG, "main thread starting done");

    loop{
        thread::sleep(std::time::Duration::from_millis(SLEEP_INTERVAL));
        let mut finish = false;
        for handle in handle_vec.iter_mut() {
            if handle.is_finished() {
                warn!(LOG_TAG, "there is a dead thread handle, main thread terminating");
                finish = true;
                break;
            }
        }

        if finish {
            break;
        }
    }
}
