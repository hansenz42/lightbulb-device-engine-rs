use std::{borrow::Borrow, cell::RefCell, rc::Rc, sync::mpsc::Sender};

use crate::util::json;
use crate::{
    common::error::DriverError,
    driver::modbus::{modbus_bus::ModbusBus, modbus_do_controller_coil::ModbusDoControllerCoil},
    entity::dto::{device_meta_info_dto::DeviceMetaInfoDto, device_state_dto::StateToDeviceControllerDto},
};

pub fn make(
    device_info: &DeviceMetaInfoDto,
    modbus_ref: &Rc<RefCell<ModbusBus>>,
    report_tx: Sender<StateToDeviceControllerDto>,
) -> Result<ModbusDoControllerCoil, DriverError> {
    let unit = json::get_config_int(&device_info.config, "unit")?;
    let output_num = json::get_config_int(&device_info.config, "num")?;
    let obj = ModbusDoControllerCoil::new(
        device_info.device_id.as_str(),
        unit.try_into().map_err(|e| {
            DriverError(format!(
                "device factory: cannot convert unit to int, err: {e}"
            ))
        })?,
        output_num.try_into().map_err(|e| {
            DriverError(format!(
                "device factory: cannot convert num to int, err: {e}"
            ))
        })?,
        Rc::clone(modbus_ref),
        report_tx,
    );
    Ok(obj)
}
