//! 测试用设备文件
use crate::{entity::bo::device_config_bo::{DummyConfigBo, ConfigBo}, common::error::DriverError};
use super::super::traits::device::Device;
use async_trait::async_trait;
use serde_json::Value;
use crate::{info};

const LOG_TAG: &str = "main";

#[derive(Debug)]
pub struct DummyDevice {

}

#[async_trait]
impl Device for DummyDevice {
    fn get_category(&self) -> (String, String) {
        ("dummy".to_string(), "dummy".to_string())
    }

    fn get_device_id(&self) -> String {
        "dummy".to_string()
    }

    async fn cmd(&self, action: &str, param: Value) -> Result<(), DriverError> {
        info!(LOG_TAG, "dummy device cmd: {}, param: {}", action, param);
        Ok(())
    }
}

impl DummyDevice {
    pub fn new(device_id: String) -> Self {
        Self{}
    }
}