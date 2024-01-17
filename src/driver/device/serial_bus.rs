//! 串口设备总线
//! 设计
//! - 

use std::sync::mpsc::Sender;
use tokio_serial::SerialStream;
use tokio_util::{codec::{Decoder, Encoder}, bytes::{self, Buf}};

use crate::{
    driver::traits::{master::Master, device::Device}, 
    entity::bo::{device_state_bo::DeviceStateBo, device_config_bo::ConfigBo},
    common::error::DriverError
};

pub struct SerialBus {
    device_id: String,
    device_type: String,
    device_class: String,
    serial_port: String,
    baudrate: u32,
    upward_channel: Option<Sender<DeviceStateBo>>,
}

impl SerialBus {
    pub fn new(device_id: String, serial_port: String, baudrate: u32) -> SerialBus {
        SerialBus {
            device_id,
            device_class: "bus".to_string(),
            device_type: "serial".to_string(),
            serial_port,
            baudrate,
            upward_channel: None,
        }
    }
}

struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(n) = buf.iter().position(|b| *b == b'\n') {
            let line = buf.split_to(n);
            buf.advance(1);
            Ok(Some(String::from_utf8(line.to_vec()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

impl Master for SerialBus {}

impl Device for SerialBus {

    fn get_category(&self) -> (String, String) {
        (self.device_class.clone(), self.device_type.clone())
    }

    fn get_device_id(&self) -> String {
        self.device_id.clone()
    }

    fn set_upward_channel(&mut self, sender: Sender<DeviceStateBo>) -> Result<(), DriverError> {
        self.upward_channel = Some(sender);
        Ok(())
    }

    fn get_upward_channel(&self) -> Option<Sender<DeviceStateBo>> {
        None
    }
}