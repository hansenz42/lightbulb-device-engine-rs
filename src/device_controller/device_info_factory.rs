//! create device info

use std::collections::HashMap;

use serde_json::Value;

use crate::{common::error::{DeviceServerError, DriverError, ServerErrorCode}, debug, entity::dto::device_info_dto::DeviceMetaInfoDto, error, info, trace, warn};

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
        let device_config_json: Value =
            serde_json::from_str(&device_po.config).map_err(|e| {
                DeviceServerError{
                    code: ServerErrorCode::DeviceInfoError,
                    msg: format!("error parsing device config json, error msg: {}", e)
                }
            })?;
        let master_device_id = device_config_json["master_device_id"]
            .as_str()
            .map(|s| s.to_string());

        // 1. make device info
        let device_info = DeviceMetaInfoDto {
            device_id: device_po.device_id.clone(),
            device_type: device_po.device_type.clone(),
            master_device_id: master_device_id,
            config: device_config_json.clone(),
            status: DeviceStatusEnum::NotInitialized,
        };

        // 2. put into device map
        let _ = ret.insert(device_po.device_id.clone(), device_info);
    }
    Ok(ret)
}