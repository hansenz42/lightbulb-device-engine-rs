use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
    thread,
    borrow::{Borrow, BorrowMut},
};

use crate::{
    entity::dto::{
        device_meta_info_dto::{DeviceMetaInfoDto, DeviceStatusEnum},
        device_state_dto::DeviceStateDto,
    },
    mqtt_client::client::MqttClient,
};

use crate::{debug, error, info, trace, warn};

const LOG_TAG: &'static str = "reporting_thread";

/// upward reporting thread
/// used to report device state to upward controllers
/// the thread using tokio runtime because mqtt client is async
pub fn reporting_thread(
    state_report_rx: mpsc::Receiver<DeviceStateDto>,
    mqtt_client: Arc<Mutex<MqttClient>>,
    device_info_map: Arc<Mutex<HashMap<String, DeviceMetaInfoDto>>>,
) {
    thread::spawn(move || loop {
        info!(LOG_TAG, "waiting for device reporting message");
        let message = state_report_rx.recv();
        match message {
            Ok(device_state_dto) => {
                info!(LOG_TAG, "report message to mqtt: {:?}", &device_state_dto);
                let device_id = device_state_dto.device_id.clone();
                let device_state_copy = device_state_dto.state.clone();
                // 1 update device state and mark device status to "active"
                {
                    let mut map_guard = device_info_map.lock().unwrap();
                    if let Some(device_info) = map_guard.borrow_mut().get_mut(device_id.as_str()) {
                        device_info.state = device_state_copy;
                        device_info.status = DeviceStatusEnum::ACTIVE;
                    }
                }
                // 2 send out mqtt message
                {
                    let mqtt_guard = mqtt_client.lock().unwrap();
                    mqtt_guard
                        .publish_status(device_state_dto)
                        .expect("cannot publish upward message to mqtt");
                }
            }
            Err(e) => {
                warn!(
                    LOG_TAG,
                    "report thread exiting: device report channel error, msg: {}", e
                );
                return;
            }
        }
    });
}
