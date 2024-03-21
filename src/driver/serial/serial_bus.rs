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

use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}, thread};
use actix_web::http::uri::Port;
use std::sync::mpsc::Sender;
use futures::{stream::SplitSink, SinkExt, StreamExt};
use super::{entity::{SerialDataBo, SerialThreadCommand}, traits::SerialMountable};
use crate::{
	entity::bo::{device_state_bo::DeviceStateBo, device_config_bo::ConfigBo},
	common::error::DriverError
};
use super::serial_thread::run_loop;

const LOG_TAG: &str = "serial_bus.rs | serial bus controller";

pub struct SerialBus {
	device_id: String,
	device_type: String,
	device_class: String,
	serial_port: String,
	baudrate: u32,
	// upward channel reporting device state
	upward_channel: Option<Sender<DeviceStateBo>>,
	// command sending channel to serial port
	command_channel_tx: Option<tokio::sync::mpsc::Sender<SerialThreadCommand>>,
	// registered listener of the serial port
	listeners: Vec<Box<dyn SerialMountable + Send>>,
	// thread running handle
	thread_handle: Option<thread::JoinHandle<()>>
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
			command_channel_tx: None,
			listeners: Vec::new(),
			thread_handle: None
		}
	}

	/// start the thread
	/// - clone the listeners into new thread
	/// - make command transmitting channel
	/// - run thread
	pub fn start(&mut self) -> Result<(), DriverError>{

		// create the command channel
		let (command_tx, command_rx) = tokio::sync::mpsc::channel(100);
		self.command_channel_tx = Some(command_tx);

		self.thread_handle = Some(thread::spawn(move || {

			// create listeners
			let listners: Vec<RefCell<Box<dyn SerialMountable + Send>>> = self.listeners.iter().map(|listener| {
				let listener = *listener.clone();
				RefCell::new(listener)
			}).collect();

			let _ = run_loop(self.serial_port.as_str(), self.baudrate, command_rx, self.listeners);
		}));

		Ok(())
	}

	/// TODO: send data to serial
	pub fn send_data(&mut self, data: SerialDataBo) -> Result<(), DriverError> {
		Ok(())
	}

	pub fn stop(&mut self) -> Result<(), DriverError> {
		Ok(())
	}	

}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::common::logger::init_logger;

	fn set_env(){
		let _ = init_logger();
	}

	#[test]
	fn test_new() {
		set_env();
		let serial_bus = SerialBus::new("serial_bus_1", "/dev/ttyUSB1", 9600);
	}

	#[test]
	fn test_mount_dummy_serial_listener(){
		set_env();
	}
}