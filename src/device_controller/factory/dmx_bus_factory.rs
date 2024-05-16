use std::sync::mpsc::Sender;

use serde_json::Value;
use crate::entity::dto::device_meta_info_dto::DeviceMetaInfoDto;
use crate::entity::dto::device_state_dto::DeviceStateDto;
use crate::{common::error::DriverError, driver::dmx::dmx_bus::DmxBus};
use crate::util::json;

pub fn make(device_info: &DeviceMetaInfoDto, report_tx: Sender<DeviceStateDto>) -> Result<DmxBus, DriverError> {
    let serial_port = json::get_str(&device_info.config, "serial_port")?;
    let obj = DmxBus::new(
        device_info.device_id.as_str(),
        serial_port.as_str(),
        report_tx
    ); 
    Ok(obj)
}