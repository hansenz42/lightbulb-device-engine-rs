//! 工厂通用特征
use crate::{entity::bo::device_config_bo::DeviceConfigBo, common::error::DeviceServerError};

pub trait Factory {
    type Product;

    fn create(&self, config_bo: DeviceConfigBo) -> Result<Self::Product, DeviceServerError>;
}