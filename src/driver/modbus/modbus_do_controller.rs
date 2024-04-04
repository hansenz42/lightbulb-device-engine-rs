use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::modbus_bus::ModbusBus;
use super::prelude::*;
use super::traits::{ModbusCaller, ModbusDoControllerCaller};
use std::sync::mpsc;
use super::entity::{ModbusThreadCommandEnum, WriteMultiBo, WriteSingleBo};
use crate::common::error::DriverError;
use crate::device_controller::entity::device_enum::DeviceRefEnum;
use crate::driver::traits::{Refable, SetRef};


/// modbus digital output controller
/// - record port state
/// - compare incoming data with recorded state, if different, then send to dmx interface
pub struct ModbusDoController {
    device_id: String,
    unit: ModbusUnitSize,
    output_num: ModbusAddrSize,
    mount_port_map: HashMap<ModbusAddrSize, Box<dyn ModbusDoControllerCaller + Send>>,
    port_state_vec: Vec<bool>,
    // the type here should be modbus
    modbus_ref: Rc<RefCell<ModbusBus>>
}

impl Refable for ModbusDoController{}

impl ModbusCaller for ModbusDoController {
    fn get_unit(&self) -> ModbusUnitSize {
        self.unit
    }

    fn get_port_num(&self) -> ModbusAddrSize {
        self.output_num
    }

    fn write_one_port(&self, address: ModbusAddrSize, value:bool) -> Result<(), DriverError> {
        // check address range 
        if address >= self.output_num {
            return Err(DriverError(format!("ModbusDoController: writing address out of range, device_id: {}, address: {}, value: {}", self.device_id, address, value)));
        }

        // check if the value is different 
        let port_state = self.port_state_vec[address as usize];
        if port_state != value {
            if let Ok(modbus_ref )= self.modbus_ref.try_borrow() {
                let _ = modbus_ref.write_single_port(self.unit, address, value)?;
            } else {
                return Err(DriverError(format!("ModbusDoController: failed to borrow modbus_ref")));
            }
        }
        Ok(())
    }

    fn write_multi_port(&self, address: ModbusAddrSize, values: &[bool]) -> Result<(), DriverError> {
        // check address range 
        if address + values.len() as ModbusAddrSize > self.output_num {
            return Err(DriverError(format!("ModbusDoController: writing address out of range, device_id: {}, address: {}, values: {:?}", self.device_id, address, values)));
        }
        
        // check if the values are different 
        let len = values.len();
        let port_state_slice = &self.port_state_vec[address as usize..(address as usize + len)];
        let mut is_diff = false;
        for i in 0..len {
            if port_state_slice[i as usize] != values[i as usize] {
                is_diff = true;
                break;
            }
        }

        if is_diff {
            if let Ok(modbus_ref) = self.modbus_ref.try_borrow() {
                let _ = modbus_ref.write_multi_port(self.unit, address, values)?;
            } else {
                return Err(DriverError(format!("ModbusDoController: failed to borrow modbus_ref")));
            }
        }

        Ok(())
    }

}

impl ModbusDoController {
    pub fn new(
        device_id: &str,
        unit: ModbusUnitSize, 
        output_num: ModbusAddrSize, 
        modbus_ref: Rc<RefCell<ModbusBus>>,
    ) -> Self {
        ModbusDoController {
            device_id: device_id.to_string(),
            unit,
            output_num,
            mount_port_map: HashMap::new(),
            port_state_vec: vec![false; output_num as usize],
            modbus_ref: modbus_ref
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use super::super::modbus_bus::ModbusBus;
    use crate::common::logger::init_logger;

    // // write single port 
    // #[test]
    // fn test_write() {
    //     env::set_var("mode", "dummy");
    //     let _ = init_logger();
        
    //     let mut modbus = ModbusBus::new("test_device_id", "/dev/null", 9600);

    //     modbus.start().unwrap();

    //     let mut controller = ModbusDoController {
    //         device_id: "test".to_string(),
    //         unit: 1,
    //         output_num: 10,
    //         mount_port_map: HashMap::new(),
    //         port_state_vec: vec![false; 10],
    //         modbus_enum: &modbus
    //     };

    //     let result = controller.write_one_port(0, true);

    //     println!("{:?}", result);

    //     // wait for 10 sec
    //     std::thread::sleep(std::time::Duration::from_secs(10));
    // }

    // #[test]
    // // write multiple ports
    // fn test_write_multi() {
    //     env::set_var("mode", "dummy");
    //     let mut modbus = ModbusBus::new("test_device_id", "/dev/null", 9600);

    //     modbus.start().unwrap();

    //     let mut controller = ModbusDoController {
    //         device_id: "test".to_string(),
    //         unit: 1,
    //         output_num: 10,
    //         mount_port_map: HashMap::new(),
    //         port_state_vec: vec![false; 10],
    //         modbus_enum: &modbus
    //     };

    //     let result = controller.write_multi_port(0, &[true, false, true]);
    //     println!("{:?}", result);

    //     // wait for 10 sec
    //     std::thread::sleep(std::time::Duration::from_secs(10));
    // }
}