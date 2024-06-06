use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use super::modbus_do_controller_coil::ModbusDoControllerCoil;
use super::prelude::*;
use super::traits::{ModbusCaller, ModbusDoControllerCaller};
use crate::common::error::DriverError;
use crate::driver::traits::{Commandable, ReportUpward};
use crate::entity::dto::device_command_dto::DeviceCommandDto;
use crate::entity::dto::device_report_dto::DeviceReportDto;
use crate::entity::dto::device_state_dto::{StateToDeviceControllerDto, DoStateDto, StateDtoEnum};
use crate::{info, warn};

const DEVICE_CLASS: &str = "operable"; 
const DEVICE_TYPE: &str = "modbus_do_port";
const LOG_TAG : &str = "modbus_do_port";

pub struct ModbusDoPort {
    device_id: String,
    address: ModbusAddrSize,
    controller_ref: Rc<RefCell<ModbusDoControllerCoil>>,
    report_tx: Sender<StateToDeviceControllerDto>,
    on: bool,
    error_msg: Option<String>,
    error_timestamp: Option<u64>,
    last_update: Option<u64>,
}

impl ModbusDoPort {
    pub fn new(
        device_id: &str,
        address: ModbusAddrSize,
        controller_ref: Rc<RefCell<ModbusDoControllerCoil>>,
        report_tx: Sender<StateToDeviceControllerDto>,
    ) -> Self {
        ModbusDoPort {
            device_id: device_id.to_string(),
            address,
            controller_ref,
            report_tx,
            on: false,
            error_msg: None,
            error_timestamp: None,
            last_update: None,
        }
    }
}

impl ReportUpward for ModbusDoPort {
    fn get_upward_channel(&self) -> &Sender<StateToDeviceControllerDto> {
        return &self.report_tx;
    }

    fn report(&self) -> Result<(), DriverError> {
        let state_dto = DoStateDto {
            on: self.on,
        };
        self.notify_upward(StateToDeviceControllerDto {
            device_id: self.device_id.clone(),
            device_class: DEVICE_CLASS.to_string(),
            device_type: DEVICE_TYPE.to_string(),
            status: DeviceReportDto{
                error_msg: self.error_msg.clone(),
                error_timestamp: self.error_timestamp,
                last_update: self.last_update,
                state: StateDtoEnum::Do(state_dto),
                active: true
            }
        })?;
        Ok(())
    }
}

impl Commandable for ModbusDoPort {
    fn cmd (&mut self, dto: DeviceCommandDto) -> Result<(), DriverError> {
        if dto.action == "on" {
            self.on = true;
            self.write(true)?;
        } else if dto.action == "off" {
            self.on = false;
            self.write(false)?;
        } else {
            return Err(DriverError(format!("invalid action for ModbusDoPort: {}", dto.action)));
        }
        Ok(())
    }
}

impl ModbusDoControllerCaller for ModbusDoPort {
    fn get_address(&self) -> ModbusAddrSize {
        self.address
    }

    fn write(&self, value: bool) -> Result<(), DriverError> {
        let dummy = env::var("dummy").unwrap_or("false".to_string());
        if let Ok(mut controller) = self.controller_ref.try_borrow_mut() {
            if dummy == "true" {
                info!(LOG_TAG, "**DUMMY MODE** ModbusDoPort: write dummy, address={}, value={}", self.address, value);
            } else {
                controller.write_one_port(self.address, value)?;
            }
        } else {
            return Err(DriverError(format!(
                "ModbusDoPort: controller borrow failed, cannot write data, device_id={}",
                &self.device_id
            )));
        }
        self.report()?;
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
        // env::set_var("dummy", "true");
        // let modbus = ModbusBus::new("test", "/dev/null", 9600);
        // let controller = ModbusDoController::new(
        //     "test", 1, 8, Rc::new(modbus)
        // );
        // let port = ModbusDoPort::new("test", 1, Rc::new(controller));

        // port.write(true).unwrap();
    }
}
