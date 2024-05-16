use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use super::modbus_do_controller::ModbusDoController;
use super::prelude::*;
use super::traits::{ModbusCaller, ModbusDoControllerCaller};
use crate::common::error::DriverError;
use crate::driver::traits::ReportUpward;
use crate::entity::dto::device_state_dto::{DeviceStateDto, DoStateDto, StateDtoEnum};

const DEVICE_CLASS: &str = "operable"; 
const DEVICE_TYPE: &str = "modbus_do_port";

pub struct ModbusDoPort {
    device_id: String,
    address: ModbusAddrSize,
    controller_ref: Rc<RefCell<ModbusDoController>>,
    report_tx: Sender<DeviceStateDto>,
    on: bool
}

impl ModbusDoPort {
    pub fn new(
        device_id: &str,
        address: ModbusAddrSize,
        controller_ref: Rc<RefCell<ModbusDoController>>,
        report_tx: Sender<DeviceStateDto>,
    ) -> Self {
        ModbusDoPort {
            device_id: device_id.to_string(),
            address,
            controller_ref,
            report_tx,
            on: false
        }
    }
}

impl ReportUpward for ModbusDoPort {
    fn get_upward_channel(&self) -> &Sender<DeviceStateDto> {
        return &self.report_tx;
    }

    fn report(&self) -> Result<(), DriverError> {
        let state_dto = DoStateDto {
            on: self.on,
        };
        self.notify_upward(DeviceStateDto {
            device_id: self.device_id.clone(),
            device_class: DEVICE_CLASS.to_string(),
            device_type: DEVICE_TYPE.to_string(),
            state: StateDtoEnum::Do(state_dto),
        })?;
        Ok(())
    }
}

impl ModbusDoControllerCaller for ModbusDoPort {
    fn get_address(&self) -> ModbusAddrSize {
        self.address
    }

    fn write(&self, value: bool) -> Result<(), DriverError> {
        if let Ok(controller) = self.controller_ref.try_borrow() {
            controller.write_one_port(self.address, value);
        } else {
            return Err(DriverError(format!(
                "ModbusDoPort: controller borrow failed, cannot write data, device_id={}",
                &self.device_id
            )));
        }
        self.report();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::super::modbus_bus::ModbusBus;
    use super::*;
    use std::env;
    use std::sync::mpsc;

    #[test]
    fn test_modbus_do_port_new() {
        // env::set_var("mode", "dummy");
        // let modbus = ModbusBus::new("test", "/dev/null", 9600);
        // let controller = ModbusDoController::new(
        //     "test", 1, 8, Rc::new(modbus)
        // );
        // let port = ModbusDoPort::new("test", 1, Rc::new(controller));

        // port.write(true).unwrap();
    }
}
