use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::{
    entity::dto::{
        device_report_dto::DeviceReportDto, server_state_dto::ServerStateDto
    },
    mqtt_client::client::MqttClient,
};

use super::super::
    entity::device_po::DevicePo
;
use crate::entity::dto::device_meta_info_dto::DeviceMetaInfoDto;
use crate::{debug, error, info, trace, warn};

const LOG_TAG: &'static str = "device_manager_threads";

/// device heartbeating thread
/// send heartbeating message with device info to flow server at a regular interval
pub fn heartbeating_thread(
    beat_interval_millis: u64,
    device_info_map: Arc<Mutex<HashMap<String, DeviceMetaInfoDto>>>,
    device_config_map: HashMap<String, DevicePo>,
    mqtt_client: Arc<Mutex<MqttClient>>,
) {
    let _ = thread::spawn(move || {
        info!(LOG_TAG, "heartbeating thread starting");
        loop {
            // 1. make device report message
            let mut report_dto_map: HashMap<String, DeviceReportDto> = HashMap::new();
            {
                let map_guard = device_info_map.lock().unwrap();
                for (device_id, device_info) in map_guard.iter() {
                    let report_dto = DeviceReportDto::from_device_meta_info(device_info);
                    report_dto_map.insert(device_id.clone(), report_dto);
                }
            }

            // 2. make server state, and send heartbeating message
            let server_state = ServerStateDto {
                device_config: device_config_map.clone(),
                device_status: report_dto_map,
            };
            info!(
                LOG_TAG,
                "heartbeating thread: send server state, msg len: {}",
                server_state.device_status.len()
            );
            {
                let mqtt_guard = mqtt_client.lock().unwrap();
                let ret = mqtt_guard.publish_heartbeat(server_state);
                match ret {
                    Ok(_) => {}
                    Err(e) => {
                        error!(LOG_TAG, "heartbeating thread: cannot publish server state to mqtt, will try in next beat, error msg: {}", e);
                    }
                }
            }

            // 3. sleep for beat_interval
            thread::sleep(std::time::Duration::from_millis(beat_interval_millis));
        }
    });
}
