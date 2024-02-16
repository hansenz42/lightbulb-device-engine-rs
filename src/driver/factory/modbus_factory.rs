//! modbus 控制器工厂
use super::super::device::modbus::ModbusBus;
use super::traits::Factory;
use crate::entity::bo::device_config_bo::{ConfigBo, ModbusConfigBo};
use crate::common::error::{DeviceServerError, ServerErrorCode};
use crate::driver::traits::device::Device;

pub struct ModbusFactory {}

impl Factory for ModbusFactory {

    fn create_obj(&self, device_id: &str, config_bo: ConfigBo) -> Result<Box<dyn Device + Sync + Send>, DeviceServerError> {
        match config_bo {
            ConfigBo::Modbus(config) => {
                Ok(Box::new(ModbusBus::new(device_id, config.serial_port.as_str(), config.baudrate)))
            }
            _ => {
                Err(DeviceServerError{
                    code: ServerErrorCode::DeviceConfigError,
                    msg: format!("modbus 设备类型配置错误")
                })
            }
        }
    }

    fn get_type(&self) -> String {
        "modbus".to_string()
    }

    fn transform_config(&self, device_config_json: String) -> Result<ConfigBo, DeviceServerError> {
        let config_bo: ModbusConfigBo = serde_json::from_str(&device_config_json).map_err(
            |err| DeviceServerError {
                code: ServerErrorCode::DeviceConfigError,
                msg: format!("设备配置文件错误: {}", err)
            }
        )?;
        Ok(ConfigBo::Modbus(config_bo))
    }
}

impl ModbusFactory {
    pub fn new() -> Self {
        ModbusFactory {}
    }
}