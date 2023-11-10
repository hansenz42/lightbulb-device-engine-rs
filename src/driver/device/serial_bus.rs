//! 串口设备总线

pub struct SerialBus {
    device_id: String,
    device_type: String,
    device_class: String,
    serial_port: String,
    baudrate: u32
}

impl SerialBus {
    pub fn new(device_id: String, serial_port: String, baudrate: u32) -> SerialBus {
        SerialBus {
            device_id,
            device_class: "bus".to_string(),
            device_type: "serial".to_string(),
            serial_port,
            baudrate
        }
    }
}