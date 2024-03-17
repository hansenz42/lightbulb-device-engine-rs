use super::prelude::*;


// dmx 线程指令对象，用于给线程下达指令用
#[derive(Debug)]
pub enum DmxThreadCommandEnum {
    // 向端口下发数据
    SetChannel(SetChannelBo),

    // 停止线程并关闭端口
    Stop,
}

// 指令：写单个线圈
#[derive(Debug)]
pub struct SetChannelBo {
    pub channels: [DmxValue; DmxChannelLen]
}