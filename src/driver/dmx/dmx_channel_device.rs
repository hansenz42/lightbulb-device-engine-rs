use crate::common::error::DriverError; 
use super::prelude::*;
use super::traits::DmxCaller;
use super::dmx_bus::DmxBus;


/// channelled device that send data to modbus
pub struct DmxChannelDevice <'a> {
    device_id: String,
    address: DmxAddress,
    channel_num: DmxAddress,
    value: Vec<DmxValue>,
    dmx_bus: &'a mut DmxBus
}

impl DmxCaller for DmxChannelDevice<'_> {
    fn set_channel(&mut self, channel: DmxAddress, value: DmxValue) -> Result<(), DriverError> {
        // check if the channel is out of range
        if channel > self.channel_num {
            return Err(DriverError(format!("channelled device set_channel failed, channel out of range, channel = {}, channel_num = {}, device_id = {}", channel, self.channel_num, self.device_id)));
        }
        // update data in vec
        self.value[channel as usize] = value;
        self.dmx_bus.set_channel(self.address + channel, value)?;
        Ok(())
    }
}

impl<'a> DmxChannelDevice<'a> {
    fn new(device_id: &str, address: DmxAddress, channel_num: DmxAddress, dmx_bus: &'a mut DmxBus) -> Self {
        DmxChannelDevice {
            device_id: device_id.to_string(),
            address,
            channel_num,
            value: vec![0; channel_num as usize],
            dmx_bus
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::logger::init_logger;
    use std::thread;
    use std::env;

    fn set_env() {
        env::set_var("dummy", "true");
        let _  = init_logger();
    }

    /// test send data to dmx bus
    #[test]
    fn test_channel_device() {
        set_env();

        let mut dmx_bus = DmxBus::new("test", "test");
        let _ = dmx_bus.start(); 
        let mut dmx_channel_device = DmxChannelDevice::new("test", 1, 255, &mut dmx_bus);
        let _ = dmx_channel_device.set_channel(3, 255);

        thread::sleep(std::time::Duration::from_secs(60));
    }
}