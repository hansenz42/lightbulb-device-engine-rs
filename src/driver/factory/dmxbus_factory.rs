//! modbus 控制器工厂
use super::super::device::dmx_bus::DmxBus;
use super::traits::Factory;
use crate::driver::traits::device::Device;
use crate::entity::bo::device_config_bo::{ConfigBo};
use crate::common::error::{DeviceServerError, ServerErrorCode};

pub struct DmxBusFactory {}

impl Factory for DmxBusFactory {
    fn create_obj(&self, device_id: &str, config_bo: ConfigBo) -> Result<Box<dyn Device + Sync + Send>, DeviceServerError> {
        match config_bo {
            ConfigBo::DmxBus(config) => {
                let serial_port = config.ftdi_serial;
                Ok(Box::new(DmxBus::new(device_id, serial_port.as_str())))
            }
            _ => {
                Err(DeviceServerError{
                    code: ServerErrorCode::DeviceConfigError,
                    msg: "创建 dmx 设备失败，配置类型错误".to_string()
                })
            }
        }
    }

    fn get_type(&self) -> String {
        "dmx".to_string()
    }

    fn transform_config(&self, device_config_json: String) -> Result<ConfigBo, DeviceServerError> {
        let config_bo: crate::entity::bo::device_config_bo::DmxBusConfigBo = serde_json::from_str(&device_config_json).map_err(
            |err| DeviceServerError {
                code: ServerErrorCode::DeviceConfigError,
                msg: format!("设备配置文件错误: {}", err)
            }
        )?;
        Ok(ConfigBo::DmxBus(config_bo))
    
    }
}

impl DmxBusFactory {
    pub fn new() -> Self {
        DmxBusFactory {}
    }
}