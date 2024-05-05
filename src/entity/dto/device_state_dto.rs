//! device state data transmission object

use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

// bus states

#[derive(Debug, Serialize, Deserialize)]
pub struct DmxBusStateDto {
    pub debug_channels: Vec<u8>
}

// controller states


#[derive(Debug, Serialize, Deserialize)]
pub struct DoControllerStateDto {
    // 输出端接口状态
    pub port: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiControllerStateBo {
    // 输入接口状态
    pub port: Vec<u8>,
}

// device states

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioStateDto {
    // 当前正在播放的音频流
    pub stream: Vec<AudioFilePlayingDto>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioFilePlayingDto {
    pub file_id: String,
    pub playing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelStateDto {
    // 通道地址
    pub address: u8,
    // 设备状态
    pub channels: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteStateDto {
    pub pressed: u8
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiStateDto {
    pub on: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoStateDto {
    pub on: bool
}