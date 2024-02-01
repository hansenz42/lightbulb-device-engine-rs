//! 串口设备总线
//! 设计
//! - 侦听端口使用线程循环（和 dmx 类似）
//! - 写数据直接使用 write 方法

use std::sync::mpsc::Sender;
use tokio_serial::SerialStream;
use tokio_util::{codec::{Decoder, Encoder}, bytes::{self, Buf}};
use tokio_serial::SerialPortBuilderExt;

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
    pub fn new(device_id: &str, serial_port: &str, baudrate: u32) -> SerialBus {
        SerialBus {
            device_id: device_id.to_string(),
            device_class: "bus".to_string(),
            device_type: "serial".to_string(),
            serial_port: serial_port.to_string(),
            baudrate,
            upward_channel: None,   
        }
    }

    pub async fn open(&mut self) {
        let mut port = tokio_serial::new(self.serial_port.clone(), self.baudrate).open_native_async()?;

        let mut reader = LineCodec.framed(port);
        while let Some(line) = reader.next().await {
            match line {
                Ok(line) => {
                    println!("receive line: {}", line);
                },
                Err(e) => {
                    println!("receive error: {}", e);
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::logger::init_logger;

    #[test]
    fn test_new() {
        let _ = init_logger();
        let serial_bus = SerialBus::new("serial_bus_1", "/dev/ttyUSB1", 9600);
    }
}