use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use crate::common::error::DriverError;
use crate::driver::dmx::dmx_bus::DmxBus;
use crate::driver::dmx::dmx_channel_device::DmxChannelDevice;
use crate::driver::dmx::prelude::DmxAddress;
use crate::entity::dto::device_meta_info_dto::DeviceMetaInfoDto;
use crate::entity::dto::device_state_dto::StateToDeviceControllerDto;
use crate::util::json;

pub fn make(
    device_info: &DeviceMetaInfoDto,
    dmx_bus_ref: Rc<RefCell<DmxBus>>,
    report_tx: Sender<StateToDeviceControllerDto>,
) -> Result<DmxChannelDevice, DriverError> {
    let channel_num = json::get_config_int(&device_info.config, "channel_num")?;
    let address = json::get_config_int(&device_info.config, "address")?;
    let obj = DmxChannelDevice::new(
        device_info.device_id.as_str(),
        address as DmxAddress,
        channel_num as DmxAddress,
        dmx_bus_ref,
        report_tx
    );
    Ok(obj)
}
