//! Modbus bus device class
//! Multiple input and output units can be mounted on modbus, using unit to identify
//! This class can operate modbus devices in the order of units.
//! function:
//! - Maintain a thread: a tokio environment runs in the thread for device scheduling
//! - When the thread is idle, it will poll all input devices (if any), and once the data changes, it will notify the upstream interface
//! - Write operation takes precedence over read operation   

use std::{
    cell::RefCell,
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
};

use super::prelude::*;
use super::{
    entity::{ModbusThreadCommandEnum, WriteMultiCoilDto, WriteSingleCoilDto},
    modbus_thread::*,
    prelude::ModbusAddrSize,
    traits::ModbusListener,
};
use crate::entity::dto::device_state_dto::{StateToDeviceControllerDto, StateDtoEnum};
use crate::{common::error::DriverError};
use std::collections::HashMap;
use std::sync::mpsc;
use crate::{info, warn, error, trace, debug};


const LOG_TAG : &str = "modbus_bus";


pub struct ModbusBus {
    device_id: String,
    serial_port: String,
    baudrate: u32,
    // Controller hashmap for modbus digital input
    di_controller_vec: Vec<Box<dyn ModbusListener + Send>>,
    // sender to send command to modbus outputing thread
    modbus_thread_command_tx: Option<Sender<ModbusThreadCommandEnum>>,
    report_tx: Sender<StateToDeviceControllerDto>,
}

impl ModbusBus {
    /// opens port and start the thread
    pub fn start(&mut self) -> Result<(), DriverError> {
        // create downward channel
        let (tx, rx) = mpsc::channel();

        let serial_port_clone = self.serial_port.clone();
        let baudrate = self.baudrate;
        let mut di_controller_map_ref_cell: HashMap<
            ModbusUnitSize,
            RefCell<Box<dyn ModbusListener + Send>>,
        > = HashMap::new();

        // drop all controller form di_controller_vec and push to ref_cell
        while let Some(controller) = self.di_controller_vec.pop() {
            let unit = controller.get_unit();
            di_controller_map_ref_cell.insert(unit, RefCell::new(controller));
        }

        // start running loop
        let _ = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = run_loop(
                    serial_port_clone.as_str(),
                    baudrate,
                    rx,
                    di_controller_map_ref_cell,
                )
                .await;
            });
        });

        self.modbus_thread_command_tx = Some(tx);

        info!(LOG_TAG, "modbus thread started, port: {}, baudrate: {}", &self.serial_port, baudrate);

        Ok(())
    }

    pub fn new(
        device_id: &str,
        serial_port: &str,
        baudrate: u32,
        report_tx: Sender<StateToDeviceControllerDto>,
    ) -> Self {
        Self {
            device_id: device_id.to_string(),
            serial_port: serial_port.to_string(),
            baudrate: baudrate,
            di_controller_vec: Vec::new(),
            modbus_thread_command_tx: None,
            report_tx,
        }
    }

    /// add a di controller the modbus
    /// but remember, you can only add new di controller before the thread starts
    pub fn add_di_controller(
        &mut self,
        unit: ModbusUnitSize,
        controller: Box<dyn ModbusListener + Send>,
    ) {
        self.di_controller_vec.push(controller);
    }

    pub fn write_single_port(
        &self,
        unit: ModbusUnitSize,
        addr: ModbusAddrSize,
        value: bool,
    ) -> Result<(), DriverError> {
        let command = ModbusThreadCommandEnum::WriteSingleCoil(WriteSingleCoilDto {
            unit: unit,
            address: addr,
            value: value,
        });
        let _ = self.send_command_to_thread(command)?;
        Ok(())
    }

    /// write multiple port at one time
    pub fn write_multi_port(
        &self,
        unit: ModbusUnitSize,
        addr: ModbusAddrSize,
        values: &[bool],
    ) -> Result<(), DriverError> {
        let command = ModbusThreadCommandEnum::WriteMultiCoils(WriteMultiCoilDto {
            unit: unit,
            start_address: addr,
            values: Vec::from(values),
        });
        let _ = self.send_command_to_thread(command)?;
        Ok(())
    }

    /// private function, send command to modbus thread
    fn send_command_to_thread(&self, command: ModbusThreadCommandEnum) -> Result<(), DriverError> {
        match self.modbus_thread_command_tx.as_ref() {
            Some(tx) => {
                let _ = tx.send(command).map_err(|e| {
                    DriverError(format!("ModbusBus send_command_to_thread error: {}", e))
                });
                Ok(())
            }
            None => {
                return Err(DriverError(format!(
                    "ModbusBus send_command_to_thread tx_down is None"
                )));
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
