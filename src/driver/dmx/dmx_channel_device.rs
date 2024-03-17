use crate::common::error::DriverError; 
use super::prelude::*;
use super::traits::DmxCaller;
use super::dmx_bus::DmxBus;


/// 向 dmx 通道发送消息的调用者
pub struct DmxChannelDevice <'a> {
    device_id: String,
    address: DmxAddress,
    value: DmxValue,
    dmx_bus: &'a mut DmxBus
}

impl DmxCaller for DmxChannelDevice<'_> {
    fn update_channel_to_dmx(&mut self, value: DmxValue) -> Result<(), DriverError> {
        let _ = self.dmx_bus.set_channel(self.address, value);
        Ok(())
    }
}

impl<'a> DmxChannelDevice<'a> {
    fn new(device_id: &str, address: DmxAddress, value: DmxValue, dmx_bus: &'a mut DmxBus) -> Self {
        DmxChannelDevice {
            device_id: device_id.to_string(),
            address,
            value,
            dmx_bus
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_device() {
        let mut dmx_bus = DmxBus::new("test", "test");
        let mut dmx_channel_device = DmxChannelDevice::new("test", 1, 255, &mut dmx_bus);
        let _ = dmx_channel_device.update_channel_to_dmx(255);
    }
}