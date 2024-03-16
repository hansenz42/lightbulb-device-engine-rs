use crate::common::error::DriverError;
use super::prelude::*;


/// 可挂载到 modbus 上的特征，一般是一个输入控制器
/// 可将 modbus 挂载到控制器上，并对端口发送指令
pub trait ModbusDiMountable {
    fn get_unit(&self) -> ModbusUnitSize;

    fn get_port_num(&self) -> ModbusAddrSize;

    /// 将端口挂载到 modbus 控制器上
    fn mount_port(&mut self, address: ModbusAddrSize, di_port: Box<dyn ModbusDiControllerMountable + Send>) -> Result<(), DriverError>;

    /// 从总线获取数据
    fn notify_from_bus(&mut self, address: ModbusAddrSize, messages: Vec<bool>) -> Result<(), DriverError>;

    /// 向 modbus 端口发送指令
    fn notify_port(&self, address: ModbusAddrSize, message: bool) -> Result<(), DriverError>;
}

/// 可挂载到 modbus Controller 上的设备，支持向上对 DeviceManager 上报数据
pub trait ModbusDiControllerMountable {
    fn get_address(&self) -> ModbusAddrSize;

    fn notify(&self, message: bool) -> Result<(), DriverError>;
}

/// 可挂载到 modbus 总线上的输出控制器
pub trait ModbusDoMountable {
    fn get_unit(&self) -> ModbusUnitSize;

    fn get_port_num(&self) -> ModbusAddrSize;

    fn write_one_port(&self, address: ModbusAddrSize, value:bool) -> Result<(), DriverError>;

    fn write_multi_port(&self, address: ModbusAddrSize,value: &[bool]) -> Result<(), DriverError>;
}

/// 可挂在到 modbus do controller 的设备
pub trait ModbusDoControllerMountable {
    fn get_address(&self) -> ModbusAddrSize;

    fn write_to_controller(&self, message: bool) -> Result<(), DriverError>;
}