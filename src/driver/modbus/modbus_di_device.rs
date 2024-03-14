use super::traits::ModbusControllerMountable;
use std::sync::mpsc;
use super::prelude::*;

/// modbus 上挂载的单个数字量输入接口
/// - 拥有一个上行通道，可以发送信息给 DeviceManager
struct ModbusDigitalInputPort {
    address: ModbusAddrSize,
    upward_channel: mpsc::Sender<bool>,
}

impl ModbusControllerMountable for ModbusDigitalInputPort {
    fn get_address(&self) -> ModbusAddrSize {
        self.address
    }

    fn notify(&self, message: bool) -> Result<(), crate::common::error::DriverError> {
        self.upward_channel.send(message).map_err(|e| crate::common::error::DriverError(format!("ModbusDigitalInputPort 向上行通道发送消息失败，异常: {}", e)))
    }
}