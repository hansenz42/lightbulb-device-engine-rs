//! modbus 控制器工厂
use super::super::device::dmx_bus::DmxBus;
use super::traits::Factory;
use crate::entity::bo::device_config_bo::{ConfigBo};
use crate::common::error::{DeviceServerError, ErrorCode};

pub struct DmxBusFactory {}

impl Factory for DmxBusFactory {
    type Product = DmxBus;

    fn create_obj(&self, device_id: String, config_bo: ConfigBo) -> Result<Self::Product, DeviceServerError> {
        match config_bo {
            ConfigBo::DmxBus(config) => {
                let serial_port = config.ftdi_serial;
                Ok(DmxBus::new(device_id, serial_port))
            }
            _ => {
                Err(DeviceServerError{
                    code: ErrorCode::DeviceConfigError,
                    msg: "创建 dmx 设备失败，配置类型错误".to_string()
                })
            }
        }
    }
}

impl DmxBusFactory {
    pub fn new() -> Self {
        DmxBusFactory {}
    }
}