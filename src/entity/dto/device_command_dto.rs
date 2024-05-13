use serde_json::Value;

#[derive(Debug, Clone)]
pub struct DeviceCommandDto {
    pub server_id: String,
    pub device_id: String,
    pub action: String,
    pub params: DeviceParamsEnum
}

#[derive(Debug, Clone)]
pub enum DeviceParamsEnum {
    Empty,
    Do(DoParamsDto)
}

#[derive(Debug, Clone)]
pub struct DoParamsDto {
    pub on: bool
}
