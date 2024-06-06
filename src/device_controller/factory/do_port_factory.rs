use std::{cell::RefCell, rc::Rc, sync::mpsc::Sender};

use crate::{
    common::error::DriverError,
    driver::modbus::{modbus_do_controller_coil::ModbusDoControllerCoil, modbus_do_port::ModbusDoPort},
    entity::dto::{device_meta_info_dto::DeviceMetaInfoDto, device_state_dto::StateToDeviceControllerDto},
    util::json,
};

pub fn make(
    device_info: &DeviceMetaInfoDto,
    modbus_do_controller_ref: Rc<RefCell<ModbusDoControllerCoil>>,
    report_tx: Sender<StateToDeviceControllerDto>,
) -> Result<ModbusDoPort, DriverError> {
    let address = json::get_config_int(&device_info.config, "address")?;
    let obj = ModbusDoPort::new(
        device_info.device_id.as_str(),
        address.try_into().map_err(|e| {
            DriverError(format!(
                "device factory: cannot convert address to int, err: {e}"
            ))
        })?,
        modbus_do_controller_ref,
        report_tx
    );
    Ok(obj)
}
