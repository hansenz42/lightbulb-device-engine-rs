use serde_json::Value;

/// device info bo for device info mapping
pub struct DeviceInfoBo {
    pub device_id: String,
    pub master_device_id: Option<String>,
    pub device_type: String,
    pub config: Value
}