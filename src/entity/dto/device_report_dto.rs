use serde::{Deserialize, Serialize};

use super::{device_meta_info_dto::{DeviceMetaInfoDto, DeviceStatusEnum}, device_state_dto::StateDtoEnum};

/// used for device report to device manager
/// includes device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceReportDto {
    active: bool,
    error_msg: Option<String>,
    error_timestamp: Option<f64>,
    last_update: Option<f64>,
    state: StateDtoEnum
}

impl DeviceReportDto {
    pub fn from_device_meta_info(meta_info: &DeviceMetaInfoDto) -> DeviceReportDto {
        DeviceReportDto {
            // active is according to status
            active: meta_info.status == DeviceStatusEnum::ACTIVE,
            error_msg: meta_info.error_msg.clone(),
            error_timestamp: meta_info.error_timestamp,
            last_update: meta_info.last_update,
            state: meta_info.state.clone()
        }
    }
}