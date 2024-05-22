use serde_json::Value;

use crate::entity::dto::device_state_dto::StateDtoEnum;

#[derive(Debug, PartialEq, Clone)]
pub enum DeviceStatusEnum{
    NotInitialized = 0,
    Initialized,
    ACTIVE,
    ERROR,
    OFFLINE
}

/// used for device manager tracking device status
#[derive(Debug, Clone)]
pub struct DeviceMetaInfoDto {
    pub device_id: String,
    pub master_device_id: Option<String>,
    pub device_type: String,
    pub config: Value,
    pub device_status: DeviceStatusEnum,

    // for reporting device state part 
    pub error_msg: Option<String>,
    pub error_timestamp: Option<u64>,
    pub last_update: Option<u64>,
    pub state: StateDtoEnum
}