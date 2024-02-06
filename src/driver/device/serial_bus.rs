//! 串口设备总线
//! 设计
//! - 侦听端口使用线程循环（和 dmx 类似）
//! - 多播侦听：一个 serialbus 可以被多个设备同时侦听，数据将被多播
//! - 将串口中指令和参数提取出来，并返回给侦听设备
//! - 写数据直接使用 write 方法
//! 
//! 串口协议：
//！ 0xfa   0x02  0x01    0xff ...  0xff  0xed
//！ 起始位  指令  参数长度     参数数据     结束位

use std::{rc::Rc, sync::mpsc::Sender, thread};
use actix_web::http::uri::Port;
use futures::{stream::SplitSink, SinkExt, StreamExt};
use tokio_serial::SerialStream;
use tokio_util::{
    codec::{Decoder, Encoder, Framed}, 
    bytes::{self, Buf, BufMut}
};
use tokio_serial::SerialPortBuilderExt;
use std::collections::HashMap;

use crate::{
    driver::traits::{device::Device, master::Master, serial_listener::SerialListener}, 
    entity::bo::{device_state_bo::DeviceStateBo, device_config_bo::ConfigBo},
    common::error::DriverError
};

/// 声明串口的处理器
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
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

impl Encoder<String> for LineCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: String, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.len() + 1);
        dst.put(item.as_bytes());
        dst.put_u8(b'\n');
        Ok(())
    }
}

pub struct SerialBus {
    device_id: String,
    device_type: String,
    device_class: String,
    serial_port: String,
    baudrate: u32,
    upward_channel: Option<Sender<DeviceStateBo>>,

    // 串口写数据对象
    serial_writer: Option<SplitSink<Framed<SerialStream, LineCodec>, String>>,

    // 侦听中的设备列表
    device_map: HashMap<String, Rc<dyn SerialListener> >
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
            device_map: HashMap::new(),
            serial_writer: None
        }
    }

    /// 向串口发送数据
    pub async fn send_data(&mut self, data: &str) -> Result<(), DriverError> {
        if let Some(writer) = &mut self.serial_writer {
            writer.send(data.to_string()).await.map_err(|err| DriverError(format!("串口数据发送失败: {}", err)))?;
        } else {
            return Err(DriverError("串口设备未初始化".to_string()));
        }
        Ok(())
    }

    /// 注册设备
    pub fn register_slave(&mut self, device_id: &str, device: Rc<dyn SerialListener>) -> Result<(), DriverError> {
        self.device_map.insert(device_id.to_string(), device);
        Ok(())
    }

    /// 设备解除注册
    pub fn remote_slave(&mut self, device_id: &str) -> Result<(), DriverError> {
        self.device_map.remove(device_id).ok_or(DriverError("设备未注册".to_string()))?;
        Ok(())
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

    fn start(&mut self) -> Result<(), DriverError>{
        let port = tokio_serial::new(self.serial_port.clone(), self.baudrate).open_native_async()
            .map_err(|err| DriverError(format!("串口打开失败 {}", &self.serial_port)))?;

        let (writer, reader) = LineCodec.framed(port).split();

        self.serial_writer = Some(writer);

        // 创建一个新线程，用于侦听新数据
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("PANIC: 创建 tokio 运行时失败，串口设备创建侦听线程");
            rt.block_on(async {
                let mut reader  = reader;
                while let Some(line) = reader.next().await {
                    match line {
                        Ok(data) => {
                            println!("收到串口数据: {}", data);
                            // TODO 发送并且通知已经注册的设备
                        },
                        Err(err) => {
                            println!("串口数据读取失败: {}", err);
                        }
                    }
                }
            });
        });

        Ok(())
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