//! Modbus bus device class
//! Multiple input and output units can be mounted on modbus, using unit to identify
//! This class can operate modbus devices in the order of units.
//! function:
//! - Maintain a thread: a tokio environment runs in the thread for device scheduling
//! - When the thread is idle, it will poll all input devices (if any), and once the data changes, it will notify the upstream interface
//! - Write operation takes precedence over read operation   


use std::{
    borrow::BorrowMut,
    cell::RefCell,
    error::Error,
    rc::Rc,
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
};

use super::{entity::{ModbusThreadCommandEnum, WriteMultiBo, WriteSingleBo}, modbus_thread::*, prelude::ModbusAddrSize, traits::ModbusListener};
use crate::{common::error::DriverError, driver::traits::Refable};
use crate::entity::bo::{
    device_config_bo::ConfigBo,
};
use crate::entity::dto::device_command_dto::DeviceCommandDto;
use crate::entity::dto::device_state_dto::{DeviceStateDto, StateDtoEnum};
use serde_json::Value;
use std::collections::HashMap;
use tokio_modbus::{client::Context, prelude::*, Slave};
use tokio_serial::SerialStream;
use super::prelude::*;
use std::sync::mpsc;

pub struct ModbusBus {
    device_id: String,
    serial_port: String,
    baudrate: u32,
    // Controller hashmap for modbus digital input
    di_controller_vec: Vec<Box<dyn ModbusListener + Send>>,
    // sender to send command to modbus outputing thread
    tx_down: Option<Sender<ModbusThreadCommandEnum>>
}

impl Refable for ModbusBus {}

impl ModbusBus {
    /// opens port and start the thread 
    pub fn start(&mut self) -> Result<(), DriverError> {
        // create downward channel 
        let (tx_down, rx_down) = mpsc::channel();

        let serial_port_clone = self.serial_port.clone();
        let baudrate = self.baudrate;
        let mut di_controller_map_ref_cell: HashMap<ModbusUnitSize, RefCell<Box<dyn ModbusListener + Send>>> = HashMap::new();

        // drop all controller form di_controller_vec and push to ref_cell
        while let Some(controller) = self.di_controller_vec.pop() {
            let unit = controller.get_unit();
            di_controller_map_ref_cell.insert(unit, RefCell::new(controller));
        } 

        // start running loop 
        let handle = thread::spawn(move || {
            let mut rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                run_loop(serial_port_clone.as_str(), baudrate, rx_down, di_controller_map_ref_cell).await;
            });
        });

        self.tx_down = Some(tx_down);

        Ok(())
    }

    pub fn new(device_id: &str, serial_port: &str, baudrate: u32) -> Self {
        Self {
            device_id: device_id.to_string(),
            serial_port: serial_port.to_string(),
            baudrate: baudrate,
            di_controller_vec: Vec::new(),
            tx_down: None
        }
    }

    /// add a di controller the modbus
    /// but remember, you can only add new di controller before the thread starts
    pub fn add_di_controller(&mut self, unit: ModbusUnitSize, controller: Box<dyn ModbusListener + Send>) {
        self.di_controller_vec.push(controller);
    }

    pub fn write_single_port(&self, unit: ModbusUnitSize, addr: ModbusAddrSize, value: bool) -> Result<(), DriverError> {
        let command = ModbusThreadCommandEnum::WriteSingle(WriteSingleBo{
            unit: unit,
            address: addr,
            value: value
        });
        
       let _ = self.send_command_to_thread(command)?; 
        
        Ok(())
    }

    /// write multiple port at one time
    pub fn write_multi_port(&self, unit: ModbusUnitSize, addr: ModbusAddrSize, values: &[bool])-> Result<(), DriverError> {
        let command = ModbusThreadCommandEnum::WriteMulti(WriteMultiBo{
            unit: unit,
            start_address: addr,
            values: Vec::from(values),
        });
        let _ = self.send_command_to_thread(command)?;
        Ok(())
    }

    /// private function, send command to modbus thread
    fn send_command_to_thread(&self, command: ModbusThreadCommandEnum) -> Result<(), DriverError> {
        match self.tx_down.as_ref() {
            Some(tx) => {
                let _ = tx.send(command).map_err(|e| DriverError(format!("ModbusBus send_command_to_thread error: {}", e)));
                Ok(())
            },
            None => {
                return Err(DriverError(format!("ModbusBus send_command_to_thread tx_down is None")));
            }
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::logger::init_logger;

    #[test]
    fn test_new() {
        let _ = init_logger();

    }
}
