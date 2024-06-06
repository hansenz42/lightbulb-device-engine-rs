use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::modbus_bus::ModbusBus;
use super::prelude::*;
use super::traits::{ModbusCaller, ModbusDoControllerCaller};
use std::sync::mpsc::{self, Sender};
use crate::common::error::DriverError;
use crate::driver::traits::{Refable, ReportUpward};
use crate::entity::dto::device_report_dto::DeviceReportDto;
use crate::entity::dto::device_state_dto::{StateToDeviceControllerDto, DoControllerStateDto, StateDtoEnum};

const DEVICE_CLASS: &str = "controller";
const DEVICE_TYPE: &str = "modbus_do_controller";

/// modbus digital output controller
/// - record port state
/// - compare incoming data with recorded state, if different, then send to dmx interface
pub struct ModbusDoControllerCoil {
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

impl ModbusCaller for ModbusDoControllerCoil {
    fn get_unit(&self) -> ModbusUnitSize {
        self.unit
    }

    fn get_output_num(&self) -> ModbusAddrSize {
        self.output_num
    }

    fn get_device_id(&self) -> String {
        self.device_id.clone()
    }

    fn get_port_state_vec_ref(&mut self) -> &mut Vec<bool> {
        &mut self.port_state_vec
    }

    fn set_port(&mut self, address: ModbusAddrSize, value: bool) -> Result<(), DriverError> {
        let _ = self.modbus_ref.borrow_mut().write_single_coil(self.get_unit(), address, value)?;
        Ok(())
    }

    fn set_multi_ports(
            &mut self,
            address: ModbusAddrSize,
            values: &[bool],
        ) -> Result<(), DriverError> {
        let _ = self.modbus_ref.borrow_mut().write_multi_coil(self.get_unit(), address, values)?;
        Ok(())
    }
}

impl ReportUpward for ModbusDoControllerCoil {
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

impl ModbusDoControllerCoil {
    pub fn new(
        device_id: &str,
        unit: ModbusUnitSize, 
        output_num: ModbusAddrSize, 
        modbus_ref: Rc<RefCell<ModbusBus>>,
        report_tx: Sender<StateToDeviceControllerDto>
    ) -> Self {
        ModbusDoControllerCoil {
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