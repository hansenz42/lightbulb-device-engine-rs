use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use serde_json::Value;
use crate::{common::error::DriverError, device_controller::entity::device_enum::DeviceRefEnum, driver::modbus::{modbus_bus::ModbusBus, modbus_do_controller::ModbusDoController}, entity::dto::device_info_dto::DeviceMetaInfoDto};
use crate::util::json;

pub fn make(device_info: &DeviceMetaInfoDto, modbus_ref: &Rc<RefCell<ModbusBus>>) -> Result<ModbusDoController, DriverError> {
    let unit = json::get_config_int(&device_info.config, "unit")?;
    let output_num = json::get_config_int(&device_info.config, "output_num")?;
    let obj = ModbusDoController::new(
        device_info.device_id.as_str(), 
        unit.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert unit to int, err: {e}"))
        )?, 
        output_num.try_into().map_err(
            |e| DriverError(format!("device factory: cannot convert output_num to int, err: {e}"))
        )?, 
        Rc::clone(modbus_ref)
    ); 
    Ok(obj)
}