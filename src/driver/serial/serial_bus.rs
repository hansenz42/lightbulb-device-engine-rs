//! Serial Device Port
//! Design
//! - use thread to listen to the serial port
//! - broadcasting listener：a serialbus can be listened by multiple devices, data will be broadcasted. (but normally, a serial port can only be listened once)
//! - Extract command and data, return to listener
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
        let serial_port_str = self.serial_port.clone();
        let baudrate = self.baudrate;
        let mut listeners_ref_cell_vec: Vec<RefCell<Box<dyn SerialMountable + Send>>> = Vec::new();
        // create listeners
        while let Some(listener) = self.listeners.pop() {
            listeners_ref_cell_vec.push(RefCell::new(listener));
        }

        self.thread_handle = Some(thread::spawn(move || {
            let _ = run_loop(serial_port_str.as_str(), baudrate, command_rx, listeners_ref_cell_vec);
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