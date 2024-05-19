use crate::common::error::DriverError;
use super::prelude::*;

// ================= di ====================

/// 可挂载到 modbus 上的特征，一般是一个输入控制器
/// 可将 modbus 挂载到控制器上，并对端口发送指令
pub trait ModbusListener {
    fn get_unit(&self) -> ModbusUnitSize;

    fn get_port_num(&self) -> ModbusAddrSize;

    /// mount controller to modbus 
    fn add_di_port(&mut self, address: ModbusAddrSize, di_port: Box<dyn ModbusDiControllerListener + Send>) -> Result<(), DriverError>;

    /// get data from modbus
    fn notify_from_bus(&mut self, address: ModbusAddrSize, values: Vec<bool>) -> Result<(), DriverError>;

    /// relay data to port device object
    fn notify_port(&self, address: ModbusAddrSize, values: bool) -> Result<(), DriverError>;
}

/// 可挂载到 modbus Controller 上的设备，支持向上对 DeviceManager 上报数据
pub trait ModbusDiControllerListener {
    fn get_address(&self) -> ModbusAddrSize;

    fn notify(&self, message: bool) -> Result<(), DriverError>;
}

// ================= do ====================

/// 可挂载到 modbus 总线上的输出控制器
pub trait ModbusCaller {
    fn get_unit(&self) -> ModbusUnitSize;

    fn get_port_num(&self) -> ModbusAddrSize;

    fn write_one_port(&mut self, address: ModbusAddrSize, value:bool) -> Result<(), DriverError>;

    fn write_multi_port(&mut self, address: ModbusAddrSize,value: &[bool]) -> Result<(), DriverError>;
}

/// 可挂在到 modbus do controller 的设备
pub trait ModbusDoControllerCaller {
    fn get_address(&self) -> ModbusAddrSize;

    fn write(&self, value: bool) -> Result<(), DriverError>;
}

