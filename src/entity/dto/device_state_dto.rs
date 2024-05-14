//! device state data transmission object

use std::error::Error;

use serde::{Deserialize, Serialize};

/// universal device status enum
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum StateDtoEnum {
    Empty,
    DmxBus(DmxBusStateDto),
    DoController(DoControllerStateDto),
    DiController(DiControllerStateBo),
    Audio(AudioStateDto),
    Channel(ChannelStateDto),
    Remote(RemoteStateDto),
    Di(DiStateDto),
    Do(DoStateDto),
}

/// used for device report to device manager
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceStateDto {
    // 设备 id
    pub device_id: String,
    // 设备类型
    pub device_class: String,
    // 设备二级类目
    pub device_type: String,
    // 设备状态
    pub state: StateDtoEnum
}

impl DeviceStateDto {
    pub fn to_json(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

// bus states

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DmxBusStateDto {
    pub debug_channels: Vec<u8>
}

// controller states


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoControllerStateDto {
    // 输出端接口状态
    pub port: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiControllerStateBo {
    // 输入接口状态
    pub port: Vec<u8>,
}

// device states

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioStateDto {
    // 当前正在播放的音频流
    pub stream: Vec<AudioFilePlayingDto>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioFilePlayingDto {
    pub file_id: String,
    pub playing: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelStateDto {
    // 通道地址
    pub address: u8,
    // 设备状态
    pub channels: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteStateDto {
    pub pressed: u8
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiStateDto {
    pub on: bool
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoStateDto {
    pub on: bool
}