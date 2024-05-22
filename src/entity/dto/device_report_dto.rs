use serde::{Deserialize, Serialize};

use super::{device_meta_info_dto::{DeviceMetaInfoDto, DeviceStatusEnum}, device_state_dto::StateDtoEnum};

/// used for device report to device manager
/// includes device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceReportDto {
    pub active: bool,
    pub error_msg: Option<String>,
    pub error_timestamp: Option<u64>,
    pub last_update: Option<u64>,
    pub state: StateDtoEnum
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