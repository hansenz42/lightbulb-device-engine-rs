use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::modbus_bus::ModbusBus;
use super::prelude::*;
use super::traits::{ModbusCaller, ModbusDoControllerCaller};
use std::sync::mpsc::{self, Sender};
use super::entity::{ModbusThreadCommandEnum, WriteMultiCoilDto, WriteSingleCoilDto};
use crate::common::error::DriverError;
use crate::driver::traits::{Refable, ReportUpward};
use crate::entity::dto::device_report_dto::DeviceReportDto;
use crate::entity::dto::device_state_dto::{StateToDeviceControllerDto, DoControllerStateDto, StateDtoEnum};

const DEVICE_CLASS: &str = "controller";
const DEVICE_TYPE: &str = "modbus_do_controller";

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
    modbus_ref: Rc<RefCell<ModbusBus>>,
    report_tx: Sender<StateToDeviceControllerDto>,
    error_msg: Option<String>,
    error_timestamp: Option<u64>,
    last_update: Option<u64>,
}

impl ModbusCaller for ModbusDoController {
    fn get_unit(&self) -> ModbusUnitSize {
        self.unit
    }

    fn get_port_num(&self) -> ModbusAddrSize {
        self.output_num
    }

    fn write_one_port(&mut self, address: ModbusAddrSize, value:bool) -> Result<(), DriverError> {
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

        // update port state
        self.port_state_vec[address as usize] = value;

        self.report()?;
        Ok(())
    }

    fn write_multi_port(&mut self, address: ModbusAddrSize, values: &[bool]) -> Result<(), DriverError> {
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

        // update port state
        for i in 0..len {
            self.port_state_vec[(address as usize + i) as usize] = values[i];
        }

        self.report()?;
        Ok(())
    }

}

impl ReportUpward for ModbusDoController {
    fn get_upward_channel(&self) -> &Sender<StateToDeviceControllerDto> {
        return &self.report_tx;
    }

    fn report(&self) -> Result<(), DriverError> {
        let state_dto = DoControllerStateDto {
            port: self.port_state_vec.clone()
        };
        
        self.notify_upward(StateToDeviceControllerDto{
            device_id: self.device_id.clone(),
            device_class: DEVICE_CLASS.to_string(),
            device_type: DEVICE_TYPE.to_string(),
            status: DeviceReportDto {
                error_msg: self.error_msg.clone(),
                error_timestamp: self.error_timestamp,
                last_update: self.last_update,
                active: true,
                state: StateDtoEnum::DoController(state_dto),
            }
        })?;
        Ok(())
    }
}

impl ModbusDoController {
    pub fn new(
        device_id: &str,
        unit: ModbusUnitSize, 
        output_num: ModbusAddrSize, 
        modbus_ref: Rc<RefCell<ModbusBus>>,
        report_tx: Sender<StateToDeviceControllerDto>
    ) -> Self {
        ModbusDoController {
            device_id: device_id.to_string(),
            unit,
            output_num,
            mount_port_map: HashMap::new(),
            port_state_vec: vec![false; output_num as usize],
            modbus_ref: modbus_ref,
            report_tx,
            error_msg: None,
            error_timestamp: None,
            last_update: None,
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