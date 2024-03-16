use super::prelude::*;
use super::modbus_do_controller::ModbusDoController;
use super::traits::{ModbusDoControllerMountable, ModbusDoMountable};
use crate::common::error::DriverError;

pub struct ModbusDoPort <'a> {
    device_id: String,
    address: ModbusAddrSize,
    controller_ref: &'a ModbusDoController
}

impl <'a> ModbusDoPort <'a> {
    pub fn new(device_id: &str, address: ModbusAddrSize, controller_ref: &'a ModbusDoController) -> Self {
        ModbusDoPort {
            device_id: device_id.to_string(),
            address,
            controller_ref
        }
    }
}

impl <'a> ModbusDoControllerMountable for ModbusDoPort <'a> {
    fn get_address(&self) -> ModbusAddrSize {
        self.address
    }
    
    fn write_to_controller(&self, value: bool) -> Result<(), DriverError> {
        self.controller_ref.write_one_port(self.address, value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_modbus_do_port_new() {
        let (tx, rx) = mpsc::channel();
        let controller = ModbusDoController::new("test", 1, 8, tx);
        let port = ModbusDoPort::new("test", 1, &controller);

        port.write_to_controller(true).unwrap();
        let result = rx.recv().unwrap();
        println!("{:?}", result);
    }
}