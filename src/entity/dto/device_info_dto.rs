use serde_json::Value;

use crate::entity::dto::device_state_dto::StateDtoEnum;

#[derive(Debug, PartialEq, Clone)]
pub enum DeviceStatusEnum{
    NotInitialized = 0,
    Initialized
}

/// device info bo for device info mapping
#[derive(Debug, Clone)]
pub struct DeviceMetaInfoDto {
    pub device_id: String,
    pub master_device_id: Option<String>,
    pub device_type: String,
    pub config: Value,
    pub status: DeviceStatusEnum,
    
    pub active: bool,
    pub error_msg: String,
    pub error_timestamp: f64,
    pub last_update: f64,
    pub state: StateDtoEnum
}