//! 工厂通用特征
use serde_json::Value;
use crate::driver::traits::device::Device;
use crate::{entity::{bo::device_config_bo::{DeviceCreateBo, ConfigBo}, po::device_po::DevicePo}, common::error::DeviceServerError};

pub trait Factory {
    fn create(&self, device_po: DevicePo) -> Result<Box<dyn Device + Sync + Send>, DeviceServerError> {
        let device_type = device_po.device_type;
        if self.get_type() != device_type {
            return Err(DeviceServerError{
                code: crate::common::error::ErrorCode::DeviceTypeNotSupport,
                msg: format!("当前工厂 {} 不支持该类型：{}", self.get_type(), device_type)
            })
        }

        let config_bo = self.transform_config(device_po.config)?;
        self.create_obj(device_po.device_id, config_bo)
    }

    fn create_obj(&self, device_id: String, config_bo: ConfigBo) -> Result<Box<dyn Device + Sync + Send>, DeviceServerError> ;

    // 获取当前设备的支持类型
    fn get_type(&self) -> String;

    // 将 json 格式的 config 对象转换为 ConfigBo
    fn transform_config(&self, device_config_json: String) -> Result<ConfigBo, DeviceServerError>;
}