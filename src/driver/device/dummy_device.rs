//! 测试用设备文件
use std::sync::mpsc::Sender;

use crate::{entity::bo::{device_config_bo::{DummyConfigBo, ConfigBo}, device_state_bo::DeviceStateBo}, common::error::DriverError};
use super::super::traits::device::Device;
use serde_json::Value;
use crate::{info};

const LOG_TAG: &str = "main";

#[derive(Debug)]
pub struct DummyDevice {}

impl Device for DummyDevice {
    fn set_upward_channel(&mut self, sender: Sender<DeviceStateBo>) -> Result<(), DriverError> {
        Ok(())
    }

    fn get_upward_channel(&self) -> Option<Sender<DeviceStateBo>> {
        None
    }

    fn get_category(&self) -> (String, String) {
        (String::from("dummy"), String::from("dummy"))
    }

    fn get_device_id(&self) -> String {
        String::from("dummy")
    }

    fn cmd(&mut self, action: String, param: Value) -> Result<(), DriverError> {
        info!(LOG_TAG, "dummy device cmd: {}, param: {}", action, param);
        Ok(())
    }
}

impl DummyDevice {
    pub fn new(device_id: String) -> Self {
        Self{}
    }
}