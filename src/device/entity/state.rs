//! 设备相关的实体类

/// 数字输出量控制器结构体
pub struct DoControllerStateBo {
    // 输出端接口状态
    port: Vec<u8>,
}

/// 数字输入量控制器结构体
pub struct DiControllerStateBo {
    // 输入接口状态
    port: Vec<u8>,
}

/// 音频设备状态
pub struct AudioStateBo {
    // 当前正在播放的音频流
    stream: Vec<AudioFilePlayingBo>
}

/// 当前音频的播放状态
pub struct AudioFilePlayingBo {
    file_id: String,
    playing: bool,
}

/// 带通道地址的设备状态
pub struct ChannelStateBo {
    // 通道地址
    address: u8,
    // 设备状态
    channels: Vec<u8>,
}

/// 遥控器状态
pub struct RemoteStateBo {
    pressed: u8
}

/// Di 输入设备状态
pub struct DiStateBo {
    on: u8
}

/// Do 输出设备状态
pub struct DoStateBo {
    on: u8
}