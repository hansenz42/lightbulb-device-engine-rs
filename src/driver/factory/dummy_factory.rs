//! 测试用设备 dummy 工厂
use super::super::device::dummy_device::DummyDevice;
use super::traits::Factory;
use crate::entity::bo::device_config_bo::{ConfigBo, DummyConfigBo};
use crate::common::error::{DeviceServerError, ServerErrorCode};
use crate::driver::traits::device::Device;

pub struct DummyFactory {}

impl Factory for DummyFactory {

    fn create_obj(&self, device_id: &str, config_bo: ConfigBo) -> Result<Box<dyn Device + Sync + Send>, DeviceServerError> {
        match config_bo {
            ConfigBo::Dummy(config) => {
                Ok(Box::new(DummyDevice::new(device_id)))
            }
            _ => {
                Err(DeviceServerError{
                    code: ServerErrorCode::DeviceConfigError,
                    msg: format!("dummy 设备类型配置错误")
                })
            }
        }
    }

    fn get_type(&self) -> String {
        "dummy".to_string()
    }

    fn transform_config(&self, device_config_json: String) -> Result<ConfigBo, DeviceServerError> {
        let config_bo: DummyConfigBo = serde_json::from_str(&device_config_json).map_err(
            |err| DeviceServerError {
                code: ServerErrorCode::DeviceConfigError,
                msg: format!("设备配置文件错误: {}", err)
            }
        )?;
        Ok(ConfigBo::Dummy(config_bo))
    }
}

impl DummyFactory {
    pub fn new() -> Self {
        DummyFactory {}
    }
}