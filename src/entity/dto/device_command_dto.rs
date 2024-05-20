use serde::{Deserialize, Serialize};
use serde_json::Value;

/// used for commanding device
#[derive(Debug, Clone)]
pub struct DeviceCommandDto {
    pub server_id: String,
    pub device_id: String,
    pub action: String,
    pub params: CommandParamsEnum
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandParamsEnum {
    Empty,
    Audio(AudioParamsDto)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioParamsDto {
    pub hash: String
}