//! device state data transmission object

use std::error::Error;

use serde::{Deserialize, Serialize};

use super::device_report_dto::DeviceReportDto;

/// universal device status enum
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum StateDtoEnum {
    Empty,
    DmxBus(DmxBusStateDto),
    DoController(DoControllerStateDto),
    DiController(DiControllerStateDto),
    Audio(AudioStateDto),
    Channel(ChannelStateDto),
    Remote(RemoteStateDto),
    Di(DiStateDto),
    Do(DoStateDto),
}

/// used for device report to device controller
#[derive(Debug, Serialize, Deserialize)]
pub struct StateToDeviceControllerDto {
    pub device_id: String,
    pub device_class: String,
    pub device_type: String,
    pub status: DeviceReportDto,
}

impl StateToDeviceControllerDto {
    pub fn to_json(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

// bus states

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DmxBusStateDto {
    pub channel: Vec<u8>
}

// controller states


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoControllerStateDto {
    pub port: Vec<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiControllerStateDto {
    pub port: Vec<bool>,
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