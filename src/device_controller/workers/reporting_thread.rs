use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::entity::dto::{
    device_meta_info_dto::{DeviceMetaInfoDto, DeviceStatusEnum},
    device_state_dto::DeviceStateDto,
    mqtt_dto::DeviceToMqttEnum,
};

use crate::{debug, error, info, trace, warn};

const LOG_TAG: &'static str = "reporting_thread";

/// upward reporting thread
/// used to report device state to upward controllers
/// the thread using tokio runtime because mqtt client is async
pub fn reporting_thread(
    state_report_rx: mpsc::Receiver<DeviceStateDto>,
    device_to_mqtt_tx: mpsc::Sender<DeviceToMqttEnum>,
    device_info_map: Arc<Mutex<HashMap<String, DeviceMetaInfoDto>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        info!(LOG_TAG, "waiting for device reporting message");
        let message = state_report_rx.recv();
        match message {
            Ok(dto) => {
                info!(LOG_TAG, "report message to mqtt: {:?}", &dto);
                let device_id = dto.device_id.clone();
                // 1 update device state and mark device status to "active"
                {
                    let mut map_guard = device_info_map.lock().unwrap();
                    if let Some(device_info) = map_guard.borrow_mut().get_mut(device_id.as_str()) {
                        device_info.state = dto.status.state.clone();
                        // conditionally update when data is not none
                        if !dto.status.error_msg.is_none() {
                            device_info.error_msg = dto.status.error_msg.clone();
                        }
                        if !dto.status.error_timestamp.is_none() {
                            device_info.error_timestamp = dto.status.error_timestamp.clone();
                        }
                        if !dto.status.last_update.is_none() {
                            device_info.last_update = dto.status.last_update.clone();
                        }
                        // make device status, according to device reporting dto
                        if dto.status.active == true {
                            device_info.device_status = DeviceStatusEnum::ACTIVE;
                        } else {
                            device_info.device_status = DeviceStatusEnum::ERROR;
                        }
                    }
                }
                // 2 send out mqtt message
                device_to_mqtt_tx.send(DeviceToMqttEnum::DeviceState(dto)).expect("send mqtt message error");
            }
            Err(e) => {
                warn!(
                    LOG_TAG,
                    "report thread exiting: device report channel error, msg: {}", e
                );
                return;
            }
        }
    })
}
