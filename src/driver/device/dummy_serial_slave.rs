//! 测试用串口设备
use std::sync::mpsc::Sender;

use crate::{
    common::error::DriverError, entity::bo::{
        device_config_bo::{ConfigBo, DummyConfigBo}, 
        device_state_bo::DeviceStateBo, serial_command_bo::SerialCommandBo
    }
};
use crate::driver::traits::device::Device;
use crate::driver::traits::serial_listener::SerialListener;
use serde_json::Value;
use crate::{info};

const LOG_TAG: &str = "main";

#[derive(Debug)]
pub struct DummySerialSlaveDevice {}

impl SerialListener for DummySerialSlaveDevice {
    fn notify(&self, data: SerialCommandBo) -> Result<(), DriverError>{
        info!(LOG_TAG, "dummy serial slave device notify: {:?}", data);
        Ok(())
    }
}

impl Device for DummySerialSlaveDevice {
    fn set_upward_channel(&mut self, sender: Sender<DeviceStateBo>) -> Result<(), DriverError> {
        Ok(())
    }

    fn get_upward_channel(&self) -> Option<Sender<DeviceStateBo>> {
        None
    }

    fn get_category(&self) -> (String, String) {
        (String::from("serial dummy"), String::from("serial dummy"))
    }

    fn get_device_id(&self) -> String {
        String::from("serial dummy")
    }

    fn cmd(&mut self, action: String, param: Value) -> Result<(), DriverError> {
        info!(LOG_TAG, "serial dummy device cmd: {}, param: {}", action, param);
        Ok(())
    }
}

impl DummySerialSlaveDevice {
    pub fn new(device_id: String) -> Self {
        Self{}
    }
}