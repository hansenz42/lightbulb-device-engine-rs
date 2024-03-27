//! device factory registery

use std::collections::HashMap;

use lazy_static::lazy_static;

// 设备 Type 到工厂 Factory 映射表
lazy_static! {
    static ref DEVICE_CLASS_TYPE_MAP: HashMap<String, Vec<String>> = {
        let mut m = HashMap::new();
        // m.insert("dmx_bus".to_string(), SerialBus);
        m.insert("serial_bus".to_string(), vec!["serial_bus".to_string()]);
        m.insert("audio_output".to_string(), vec!["audio_output".to_string()]);
        m
    };
}

struct DeviceFactory;

impl DeviceFactory {

}