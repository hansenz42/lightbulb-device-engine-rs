use serde_json::Value;

#[derive(Debug, PartialEq, Clone)]
pub enum DeviceStatusEnum{
    NotInitialized = 0,
    Initialized
}

/// device info bo for device info mapping
#[derive(Debug, PartialEq, Clone)]
pub struct DeviceInfoBo {
    pub device_id: String,
    pub master_device_id: Option<String>,
    pub device_type: String,
    pub config: Value,
    pub status: DeviceStatusEnum
}