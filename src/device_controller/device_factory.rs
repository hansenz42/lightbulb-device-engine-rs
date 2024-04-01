//! device factory registery

use super::factory::*;
use crate::{common::error::DriverError, driver::modbus::modbus_bus::ModbusBus};
use crate::{debug, error, info, trace, warn};
use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::HashMap;

struct DeviceFactory {
    modbus_map: HashMap<String, ModbusBus>,
}

impl DeviceFactory {
    fn new() -> DeviceFactory {
        DeviceFactory {
            modbus_map: HashMap::new(),
        }
    }

    /// make device with config map
    fn make_device_with_config_map(&mut self, config: Value) -> Result<(), DriverError> {
        // device map area
        let device_list = config.as_array().ok_or(DriverError(
            "device factory: cannot find modbus_bus in config".to_string(),
        ))?;

        for device_json in device_list {
            self.make_device_with_type(device_json)?;
        }

        Ok(())
    }

    fn make_device_with_type(&mut self, device_json: &Value) -> Result<(), DriverError> {
        let device_type = device_json["device_type"].as_str().ok_or(DriverError(
            "device factory: no device_type in config".to_string(),
        ))?;

        if device_type == "modbus_bus" {
            let modbus_bus = modbus_bus_factory::make(device_json)?;
            let device_id = device_json["device_id"].as_str().ok_or(DriverError(
                "device factory: no device_id in config".to_string(),
            ))?;
            self.modbus_map.insert(device_id.to_string(), modbus_bus);
        }

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::env;

    use super::*;
    use crate::common::logger::init_logger;

    fn set_env() {
        let _ = init_logger();
        env::set_var("mode", "dummy");
    }

    #[test]
    fn test_make_devices() {
        set_env();
        // generate a demo json
        let modbus_config_value = json!([{
            "device_id": "modbus_bus_test_1",
            "device_type": "modbus_bus",
            "config": {
                "serial_port": "/dev/ttyUSB0",
                "baudrate": 115200,
            }
        }]);
        let mut device_factory = DeviceFactory::new();
        let _ = device_factory.make_device_with_config_map(modbus_config_value);
        println!("done");
    }
}
