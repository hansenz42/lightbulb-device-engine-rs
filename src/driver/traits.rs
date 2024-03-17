use crate::{common::error::DriverError, entity::bo::device_state_bo::DeviceStateBo};
use std::sync::mpsc;

/// 提供上行数据的设备
pub trait UpwardDevice {
    fn get_upward_channel(&self) -> &mpsc::Sender<DeviceStateBo> ;

    fn notify_upward(&self, message: DeviceStateBo) -> Result<(), crate::common::error::DriverError> {
        let upward_channel = self.get_upward_channel();
        upward_channel.send(message).map_err(|e| crate::common::error::DriverError(format!("ModbusDigitalInputPort 向上行通道发送消息失败，异常: {}", e)))
    }
}