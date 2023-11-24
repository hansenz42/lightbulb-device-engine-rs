//! 设备状态实体类（一般用于设备向 device manager 通知设备状态）

use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum StateBoEnum {
    Empty,
    DmxBus(DmxBusStateBo),
    DoController(DoControllerStateBo),
    DiController(DiControllerStateBo),
    Audio(AudioStateBo),
    Channel(ChannelStateBo),
    Remote(RemoteStateBo),
    Di(DiStateBo),
    Do(DoStateBo),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceStateBo {
    // 设备 id
    pub device_id: String,
    // 设备类型
    pub device_class: String,
    // 设备二级类目
    pub device_type: String,
    // 设备状态
    pub state: StateBoEnum
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DmxBusStateBo {
    pub debug_channels: Vec<u8>
}

/// 数字输出量控制器结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct DoControllerStateBo {
    // 输出端接口状态
    pub port: Vec<u8>,
}

/// 数字输入量控制器结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct DiControllerStateBo {
    // 输入接口状态
    pub port: Vec<u8>,
}

/// 音频设备状态
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioStateBo {
    // 当前正在播放的音频流
    pub stream: Vec<AudioFilePlayingBo>
}

/// 当前音频的播放状态
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioFilePlayingBo {
    pub file_id: String,
    pub playing: bool,
}

/// 带通道地址的设备状态
#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelStateBo {
    // 通道地址
    pub address: u8,
    // 设备状态
    pub channels: Vec<u8>,
}

/// 遥控器状态
#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteStateBo {
    pub pressed: u8
}

/// Di 输入设备状态
#[derive(Debug, Serialize, Deserialize)]
pub struct DiStateBo {
    pub on: u8
}

/// Do 输出设备状态
#[derive(Debug, Serialize, Deserialize)]
pub struct DoStateBo {
    pub on: u8
}