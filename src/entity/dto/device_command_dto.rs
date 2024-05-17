use serde_json::Value;

/// used for commanding device
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
    Do(DoParamsDto),
    Audio(AudioParamsDto)
}

#[derive(Debug, Clone)]
pub struct DoParamsDto {
    pub on: bool
}

#[derive(Debug, Clone)]
pub struct AudioParamsDto {
    pub filename: String
}