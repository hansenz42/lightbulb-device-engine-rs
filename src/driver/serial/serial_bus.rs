//! Serial Device Port
//! Design
//! - use thread to listen to the serial port
//! - broadcasting listener：a serialbus can be listened by multiple devices, data will be broadcasted. (but normally, a serial port can only be listened once)
//! - Extract command and data, return to listener
//!
//! Serial Port Protocol：
//！ 0xfa   0x02     0x01      0xff ...  0xff  0xed
//！ start  command  paramlen      data        end

use super::serial_thread::run_loop;
use super::{
    entity::{SerialDataBo, SerialThreadCommand},
    traits::SerialMountable,
};
use crate::{
    common::error::DriverError,
    entity::bo::{device_config_bo::ConfigPo},
    entity::dto::device_state_dto::DeviceStateDto
};
use actix_web::http::uri::Port;
use futures::{stream::SplitSink, SinkExt, StreamExt};
use std::sync::mpsc::Sender;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

const LOG_TAG: &str = "serial_bus.rs | serial bus controller";

pub struct SerialBus {
    device_id: String,
    device_type: String,
    device_class: String,
    serial_port: String,
    baudrate: u32,
    // upward channel reporting device state
    // CAUTION: it is the std Sender
    upward_channel: Option<Sender<DeviceStateDto>>,
    // command sending channel to serial port
    // caution: it is the TOKIO Sender
    command_channel_tx: Option<tokio::sync::mpsc::Sender<SerialThreadCommand>>,
    // registered listener of the serial port
    listeners: Vec<Box<dyn SerialMountable + Send>>,
    // thread running handle
    thread_handle: Option<thread::JoinHandle<()>>,
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
            thread_handle: None,
        }
    }

    /// start the thread
    /// - clone the listeners into new thread
    /// - make command transmitting channel
    /// - run thread
    pub fn start(&mut self) -> Result<(), DriverError> {
        // create the command channel
        let (command_channel_tx, command_channel_rx) = tokio::sync::mpsc::channel(100);
        self.command_channel_tx = Some(command_channel_tx);
        let serial_port_str = self.serial_port.clone();
        let baudrate = self.baudrate;
        let mut listeners_ref_cell_vec: Vec<RefCell<Box<dyn SerialMountable + Send>>> = Vec::new();
        // create listeners
        while let Some(listener) = self.listeners.pop() {
            listeners_ref_cell_vec.push(RefCell::new(listener));
        }

        self.thread_handle = Some(thread::spawn(move || {
            let _ = run_loop(
                serial_port_str.as_str(),
                baudrate,
                command_channel_rx,
                listeners_ref_cell_vec,
            );
        }));

        Ok(())
    }

    /// register a listener
    /// CAUTION: you have to call start() before this, otherwise the listener will not work
    pub fn add_listener(&mut self, listener: Box<dyn SerialMountable + Send>) {
        self.listeners.push(listener);
    }

    /// send data to serial
    pub fn send_data(&mut self, data: SerialDataBo) -> Result<(), DriverError> {
        if let Some(tx) = self.command_channel_tx.as_mut() {
            let _ = tx
                .blocking_send(SerialThreadCommand::Write(data))
                .map_err(|e| DriverError(format!("SerialBus send data error: {}", e)))?;
        } else {
            return Err(DriverError(format!(
                "SerialBus send_data command_channel_tx is None"
            )));
        }
        Ok(())
    }

    /// send stop signal
    pub fn stop(&mut self) -> Result<(), DriverError> {
        if let Some(tx) = self.command_channel_tx.as_mut() {
            let _ = tx
                .blocking_send(SerialThreadCommand::Stop)
                .map_err(|e| DriverError(format!("SerialBus stop error: {}", e)))?;
        } else {
            return Err(DriverError(format!(
                "SerialBus stop command_channel_tx is None"
            )));
        }
        self.command_channel_tx = None;
        Ok(())
    }

    /// echo testing data
    pub fn echo_test_data(&mut self, data: Vec<u8>) -> Result<(), DriverError> {
        if let Some(tx) = self.command_channel_tx.as_mut() {
            let _ = tx
              .blocking_send(SerialThreadCommand::Echo(data))
              .map_err(|e| DriverError(format!("SerialBus echo_test_data error: {}", e)))?;
        } else {
            return Err(DriverError(format!(
                "SerialBus echo_test_data command_channel_tx is None"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::logger::init_logger;
    use std::env;
    use super::super::serial_remote_controller::SerialRemoteController;
    use std::sync::mpsc;

    fn set_env() {
        let _ = init_logger();
        env::set_var("mode", "dummy");
    }

    #[test]
    fn test_new() {
        set_env();
        let serial_bus = SerialBus::new("serial_bus_1", "/dev/ttyUSB1", 9600);
    }

    /// test with serial remote listener
    #[test]
    fn test_mount_serial_remote_listener() {
        set_env();
        let (tx, rx) = mpsc::channel();
        let mut serial_bus = SerialBus::new("serial_bus_1", "/dev/ttyUSB1", 9600);
        let mut serial_remote_controller = SerialRemoteController::new(
            "serial_remote_controller_1",
            8,
            tx,
        );
        let _ = serial_bus.add_listener(Box::new(serial_remote_controller));
        let _ = serial_bus.start();
        let _ = serial_bus.echo_test_data(vec![0xfa, 0x01, 0x01, 0xff, 0xed]);
        let state_bo = rx.recv().unwrap();
        println!("state_bo: {:?}", state_bo);

        // wait for 10 sec
        thread::sleep(std::time::Duration::from_secs(10));
    }

    #[test]
    fn test_send_data() {
        set_env();
        let mut serial_bus = SerialBus::new("serial_bus_1", "/dev/ttyUSB1", 9600);
        serial_bus.start().unwrap();
        println!("线程已启动");
        std::thread::sleep(std::time::Duration::from_secs(2));
        serial_bus
            .send_data(SerialDataBo {
                command: 0x01,
                data: vec![0x01, 0x02, 0x03],
            })
            .unwrap();
        println!("数据已发送");
        // wait for 10 sec
        std::thread::sleep(std::time::Duration::from_secs(10));
        serial_bus.stop().unwrap();
        println!("线程已停止");
        // wait for 2 sec
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
