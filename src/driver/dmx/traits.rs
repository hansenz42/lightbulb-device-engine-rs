use crate::common::error::DriverError;
use super::prelude::*;

/// 表示一个可以操作 dmx 设备的特征
pub trait DmxCaller {
    /// 向 dmx 总线更新数据
    fn set_channel(&mut self, channel: DmxAddress, value: DmxValue) -> Result<(), DriverError>;
}