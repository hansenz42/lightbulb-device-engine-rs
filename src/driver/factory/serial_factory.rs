//! modbus 控制器工厂
use super::super::device::serial_bus::SerialBus;
use super::traits::Factory;
use crate::driver::traits::device::Device;
use crate::entity::bo::device_config_bo::{ConfigBo};
use crate::common::error::{DeviceServerError, ServerErrorCode};

pub struct SerialFactory {}

impl Factory for SerialFactory {

    fn create_obj(&self, device_id: &str, config_bo: ConfigBo) -> Result<Box<dyn Device + Sync + Send>, DeviceServerError> {
        match config_bo {
            ConfigBo::SerialBus(config) => {
                Ok(Box::new(SerialBus::new(device_id, &config.serial_port, config.baudrate)))
            }
            _ => {
                Err(DeviceServerError{
                    code: ServerErrorCode::DeviceConfigError,
                    msg: "创建串口设备失败，配置类型错误".to_string()
                })
            }
        }
    }

    fn get_type(&self) -> String {
        "serial".to_string()
    }

    fn transform_config(&self, device_config_json: String) -> Result<ConfigBo, DeviceServerError> {
        let config_bo: crate::entity::bo::device_config_bo::SerialBusConfigBo = serde_json::from_str(&device_config_json).map_err(
            |err| DeviceServerError {
                code: ServerErrorCode::DeviceConfigError,
                msg: format!("设备配置文件错误: {}", err)
            }
        )?;
        Ok(ConfigBo::SerialBus(config_bo))
    }
}


impl SerialFactory {
    pub fn new() -> Self {
        SerialFactory {}
    }
}