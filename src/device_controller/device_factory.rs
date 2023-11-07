//! 设备初始化工厂，根据设备配置初始化设备
//! - 维护可以初始化的设备类列表，根据传入的参数选择要初始化的设备

use std::collections::HashMap;

use crate::driver::device::audio_output::AudioOutput;
use crate::driver::device::dmx_bus::DmxBus;
use crate::driver::device::serial_bus::SerialBus;
use lazy_static::lazy_static;

// 设备 Class 和 Type 映射表
lazy_static! {
    static ref DEVICE_CLASS_TYPE_MAP: HashMap<String, Vec<String>> = {
        let mut m = HashMap::new();
        m.insert("dmx_bus".to_string(), SerialBus);
        m.insert("serial_bus".to_string(), vec!["serial_bus".to_string()]);
        m.insert("audio_output".to_string(), vec!["audio_output".to_string()]);
        m
    };
}

struct DeviceFactory;

impl DeviceFactory {

}