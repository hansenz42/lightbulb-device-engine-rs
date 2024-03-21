//! Serial Device Port
//! 设计
//! - 侦听端口使用线程循环（和 dmx 类似）
//! - 多播侦听：一个 serialbus 可以被多个设备同时侦听，数据将被多播。（但是一般来讲，一个串口只接一个设备）
//! - 将串口中指令和参数提取出来，并返回给侦听设备
//! - 写数据直接使用 write 方法
//! 
//! Serial Port Protocol：
//！ 0xfa   0x02     0x01    0xff ...  0xff  0xed
//！ start  command  param   len      data   end

use std::{rc::Rc, sync::{mpsc::Sender, Arc, Mutex}, thread};
use actix_web::http::uri::Port;
use futures::{stream::SplitSink, SinkExt, StreamExt};
use tokio_serial::SerialPortBuilderExt;
use std::collections::HashMap;
use super::entity::SerialCommandBo;
use crate::{
	entity::bo::{device_state_bo::DeviceStateBo, device_config_bo::ConfigBo},
	common::error::DriverError
};

const LOG_TAG: &str = "serial_bus.rs | serial bus controller";


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

	/// send data to serial
	pub async fn send_data(&mut self, data: SerialCommandBo) -> Result<(), DriverError> {
		if let Some(writer) = &mut self.serial_writer {
			writer.send(data).await.map_err(|err| DriverError(format!("串口数据发送失败: {}", err)))?;
		} else {
			return Err(DriverError("串口设备未初始化".to_string()));
		}
		Ok(())
	}

	/// register
	pub fn register_slave(&mut self, device_id: &str, device: Arc<Mutex<Box<dyn SerialListener + Sync + Send>>>) -> Result<(), DriverError> {
		self.device_map.insert(device_id.to_string(), device);
		Ok(())
	}

	/// 设备解除注册
	pub fn remove_slave(&mut self, device_id: &str) -> Result<(), DriverError> {
		self.device_map.remove(device_id).ok_or(DriverError("设备未注册".to_string()))?;
		Ok(())
	}

	/// 测试从线程中对子设备发起通知
	fn test_thread_notify(&mut self, device_id: &str) -> Result<(), DriverError> {
		let device_arc = self.device_map.get(device_id).unwrap();
		let arc_clone = device_arc.clone();
		thread::spawn(move || {
			let device = arc_clone.lock().unwrap();
			let _ = device.notify(SerialCommandBo {
				command: 0x01,
				data: vec![0x01, 0x02]
			});
		});
		Ok(())
	}
}

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

	/// start the sending serial
	fn start(&mut self) -> Result<(), DriverError>{
		let port = tokio_serial::new(self.serial_port.clone(), self.baudrate).open_native_async()
			.map_err(|err| DriverError(format!("串口打开失败 {}", &self.serial_port)))?;

		let (writer, reader) = LineCodec.framed(port).split();
		self.serial_writer = Some(writer);

		let device_map_clone = self.device_map.clone();

		// create new thread
		thread::spawn(move || {
			let rt = tokio::runtime::Runtime::new().expect("PANIC: 创建 tokio 运行时失败，串口设备创建侦听线程");
			rt.block_on(async {
				let mut reader  = reader;
				while let Some(line) = reader.next().await {
					match line {
						Ok(data) => {
							println!("收到串口数据: {:?}", &data);
							// 接收到数据以后，解析数据，然后将数据发送给侦听设备
							// 发送过程，SerialListener 提供不可变引用调用，不可变引用的所属权在当前线程，实现方法为 Arc 引用计数
							
							// 为 device_map_clone 中的所有设备都发送数据
							for (_, device) in device_map_clone.iter() {
								let device_arc = device.clone();
								let lock_result = device_arc.lock();
								match lock_result {
									Ok(device) => {
										let _ = device.notify(data.clone());
									},
									Err(err) => {
										println!("串口数据读取失败: {}", err);
									}
								}
							}

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
	use crate::driver::device::dummy_serial_slave::DummySerialSlaveDevice;
	fn set_env(){
		let _ = init_logger();
	}
	#[test]
	fn test_new() {
		set_env();
		let serial_bus = SerialBus::new("serial_bus_1", "/dev/ttyUSB1", 9600);
	}

	#[test]
	fn test_decoder(){
		set_env();
		let mut codec = LineCodec;
		let bytes: &[u8] = &[0xfa, 0x02, 0x01, 0xff, 0xff, 0xed];
		let mut b_mut : bytes::BytesMut = bytes.into();
		let res = codec.decode(&mut b_mut).unwrap().unwrap();
		assert_eq!(res.command, 0x02);
		assert_eq!(res.data, vec![0xff, 0xff]);
	}

	#[test]
	fn test_encoder(){
		set_env();
		let mut codec = LineCodec;
		let serial_command = SerialCommandBo {
			command: 0x02,
			data: vec![0xff, 0xff]
		};
		let mut output = bytes::BytesMut::new();
		let res = codec.encode(serial_command, &mut output).unwrap();
		assert_eq!(output, vec![0xfa, 0x02, 0x02, 0xff, 0xff, 0xed]);
	}

	#[test]
	fn test_mount_dummy_serial_listener(){
		set_env();
		let dummy = DummySerialSlaveDevice::new("dummy_1".to_string());
		let mut serial_bus = SerialBus::new("serial_bus_1", "/dev/ttyUSB1", 9600);
		let _ = serial_bus.register_slave("dummy_1", Arc::new(Mutex::new(Box::new(dummy))));
		let _ = serial_bus.test_thread_notify("dummy_1");
	}
}