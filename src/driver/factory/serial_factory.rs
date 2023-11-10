//! modbus 控制器工厂
use super::super::device::serial_bus::SerialBus;
use super::traits::Factory;
use crate::entity::bo::device_config_bo::{ConfigBo};
use crate::common::error::{DeviceServerError, ErrorCode};

pub struct SerialFactory {}

impl Factory for SerialFactory {
    type Product = SerialBus;

    fn create_obj(&self, device_id: String, config_bo: ConfigBo) -> Result<Box<Self::Product>, DeviceServerError> {
        match config_bo {
            ConfigBo::SerialBus(config) => {
                Ok(SerialBus::new(device_id, config.serial_port, config.baudrate))
            }
            _ => {
                Err(DeviceServerError{
                    code: ErrorCode::DeviceConfigError,
                    msg: "创建串口设备失败，配置类型错误".to_string()
                })
            }
        }
    }
}


impl SerialFactory {
    pub fn new() -> Self {
        SerialFactory {}
    }
}