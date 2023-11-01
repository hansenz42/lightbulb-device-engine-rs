//! 设备状态实体类（一般用于设备向 device manager 通知设备状态）


pub trait StateBo {}

/// 数字输出量控制器结构体
#[derive(Debug)]
pub struct DoControllerStateBo {
    // 输出端接口状态
    port: Vec<u8>,
}

impl StateBo for DoControllerStateBo {}

/// 数字输入量控制器结构体
#[derive(Debug)]
pub struct DiControllerStateBo {
    // 输入接口状态
    port: Vec<u8>,
}

impl StateBo for DiControllerStateBo {}

/// 音频设备状态
#[derive(Debug)]
pub struct AudioStateBo {
    // 当前正在播放的音频流
    stream: Vec<AudioFilePlayingBo>
}

impl StateBo for AudioStateBo {}

/// 当前音频的播放状态
#[derive(Debug)]
pub struct AudioFilePlayingBo {
    file_id: String,
    playing: bool,
}

impl StateBo for AudioFilePlayingBo {}

/// 带通道地址的设备状态
#[derive(Debug)]
pub struct ChannelStateBo {
    // 通道地址
    address: u8,
    // 设备状态
    channels: Vec<u8>,
}

impl StateBo for ChannelStateBo {}

/// 遥控器状态
#[derive(Debug)]
pub struct RemoteStateBo {
    pressed: u8
}

impl StateBo for RemoteStateBo {}

/// Di 输入设备状态
#[derive(Debug)]
pub struct DiStateBo {
    on: u8
}

impl StateBo for DiStateBo {}

/// Do 输出设备状态
#[derive(Debug)]
pub struct DoStateBo {
    on: u8
}

impl StateBo for DoStateBo {}