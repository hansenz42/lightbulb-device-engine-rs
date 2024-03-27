use crate::{common::error::DriverError, entity::bo::{device_command_bo::DeviceCommandBo, device_state_bo::DeviceStateBo}};
use std::sync::mpsc;

/// the device that can send data to upward channel
pub trait UpwardSendable {
    fn get_upward_channel(&self) -> &mpsc::Sender<DeviceStateBo> ;

    fn notify_upward(&self, message: DeviceStateBo) -> Result<(), crate::common::error::DriverError> {
        let upward_channel = self.get_upward_channel();
        upward_channel.send(message).map_err(|e| crate::common::error::DriverError(format!("ModbusDigitalInputPort 向上行通道发送消息失败，异常: {}", e)))
    }
}

/// the device that operable by controller
pub trait Operable {
    /// send command to operable dvice
    fn operate(&self, message: DeviceCommandBo) -> Result<(), crate::common::error::DriverError>;
}


/// root device that controlles device interfaces
pub trait RootBus {
    fn start(&mut self) -> Result<(), DriverError>;
}