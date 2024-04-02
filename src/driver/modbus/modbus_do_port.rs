use std::rc::Rc;

use super::prelude::*;
use super::modbus_do_controller::ModbusDoController;
use super::traits::{ModbusDoControllerCaller, ModbusCaller};
use crate::common::error::DriverError;

pub struct ModbusDoPort  {
    device_id: String,
    address: ModbusAddrSize,
    controller_ref: Rc<ModbusDoController>
}

impl  ModbusDoPort  {
    pub fn new(device_id: &str, address: ModbusAddrSize, controller_ref: Rc<ModbusDoController>) -> Self {
        ModbusDoPort {
            device_id: device_id.to_string(),
            address,
            controller_ref
        }
    }
}

impl  ModbusDoControllerCaller for ModbusDoPort  {
    fn get_address(&self) -> ModbusAddrSize {
        self.address
    }
    
    fn write(&self, value: bool) -> Result<(), DriverError> {
        self.controller_ref.write_one_port(self.address, value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::mpsc;
    use super::super::modbus_bus::ModbusBus;
    use std::env;

    #[test]
    fn test_modbus_do_port_new() {
        env::set_var("mode", "dummy");
        let modbus = ModbusBus::new("test", "/dev/null", 9600);
        let controller = ModbusDoController::new(
            "test", 1, 8, Rc::new(modbus)
        );
        let port = ModbusDoPort::new("test", 1, Rc::new(controller));

        port.write(true).unwrap();
    }
}