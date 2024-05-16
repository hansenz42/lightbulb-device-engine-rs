//! create device info

use std::collections::HashMap;

use crate::{common::error::{DeviceServerError, DriverError, ServerErrorCode}, debug, entity::dto::{device_meta_info_dto::{DeviceMetaInfoDto, DeviceStatusEnum}, device_state_dto::StateDtoEnum}, error, info, trace, warn};

use super::entity::{
    device_po::DevicePo,
};

const LOG_TAG: &str = "device_info_factory";

/// make device_po_list into device_info_dto map and return
pub fn make_device_info(
    device_po_list: Vec<DevicePo>,
) -> Result<HashMap<String, DeviceMetaInfoDto>, DeviceServerError> {
    let mut ret: HashMap<String, DeviceMetaInfoDto> = HashMap::new();
    for device_po in device_po_list {
        let master_device_id = device_po.config["master_device_id"]
            .as_str()
            .map(|s| s.to_string());

        // 1. make device info
        let device_info = DeviceMetaInfoDto {
            device_id: device_po.device_id.clone(),
            device_type: device_po.device_type.clone(),
            master_device_id: master_device_id,
            config: device_po.config.clone(),
            status: DeviceStatusEnum::NotInitialized,

            error_msg: None,
            error_timestamp: None,
            last_update: None,
            state: StateDtoEnum::Empty,
        };

        // 2. put into device map
        let _ = ret.insert(device_po.device_id.clone(), device_info);
    }
    Ok(ret)
}