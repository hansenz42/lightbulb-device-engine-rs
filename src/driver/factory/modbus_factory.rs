//! modbus 控制器工厂
use super::super::device::modbus::ModbusBus;
use super::traits::Factory;
use crate::entity::bo::device_config_bo::{DeviceConfigBo, ConfigBo, ModbusConfigBo};
use crate::common::error::{DeviceServerError, ErrorCode};

pub struct ModbusFactory {

}

impl Factory for ModbusFactory {
    type Product = ModbusBus;

    fn create(&self, config_bo: DeviceConfigBo) -> Result<Self::Product, DeviceServerError> {
        let device_id = config_bo.device_id;
        match config_bo.config {
            ConfigBo::Modbus(config) => {
                let serial_port = config.serial_port;
                let baudrate = config.baudrate;
                Ok(ModbusBus::new(device_id, serial_port, baudrate))
            }
            _ => {
                Err(DeviceServerError{
                    code: ErrorCode::DeviceConfigError,
                    msg: "设备配置错误".to_string()
                })
            }
        }
        
    }
}