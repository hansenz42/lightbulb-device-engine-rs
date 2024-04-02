//! device factory registery

use super::entity::device_enum::DeviceRefEnum;
use super::factory::*;
use crate::driver::dmx::dmx_bus::DmxBus;
use crate::driver::serial::serial_bus::SerialBus;
use crate::{common::error::DriverError, driver::modbus::modbus_bus::ModbusBus};
use crate::{debug, error, info, trace, warn};
use lazy_static::lazy_static;
use serde_json::Value;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::rc::Rc;

struct DeviceFactory {
    device_map: HashMap<String, DeviceRefEnum>,
}

impl DeviceFactory {
    fn new() -> DeviceFactory {
        DeviceFactory {
            device_map: HashMap::new(),
        }
    }

    /// make device with config map
    fn make_device_with_config_map(&mut self, config_list: Value) -> Result<(), DriverError> {
        // device map area
        let device_list = config_list.as_array().ok_or(DriverError(
            "device factory: cannot find modbus_bus in config".to_string(),
        ))?;

        let mut config_list = device_list.clone();

        for device_config in config_list.iter() {

            let device_id = device_config["device_id"].as_str().ok_or(DriverError(
                "device factory: cannot find device_id in config".to_string(),
            ))?;

            if self.device_map.contains_key(device_id) {
                continue;
            }

            let device = self.make_device(device_config)?;

            self.device_map.insert(device_id.to_string(), device);
        }

        Ok(())
    }

    fn make_device(&mut self, device_json: &Value) -> Result<DeviceRefEnum, DriverError> {
        let device_type = device_json["device_type"].as_str().ok_or(DriverError(
            "device factory: no device_type in config".to_string(),
        ))?;

        if device_type == "modbus_bus" {
            let modbus_bus = modbus_bus_factory::make(device_json)?;
            Ok(DeviceRefEnum::ModbusBus(Rc::new(modbus_bus)))

        } else if device_type == "serial_bus" {
            let serial_bus = serial_bus_factory::make(device_json)?;
            Ok(DeviceRefEnum::SerialBus(Rc::new(serial_bus)))

        } else if device_type == "dmx_bus" {
            let dmx_bus = dmx_bus_factory::make(device_json)?;
            Ok(DeviceRefEnum::DmxBus(Rc::new(dmx_bus)))

        } else if device_type == "modbus_do_controller" {
            let master_device_id = device_json["master_device_id"].as_str().ok_or(
                DriverError("device factory: no master_device_id in config".to_string())
            )?;
            // get modbus master device
            let master_device_enum = self.get_master_device_enum(master_device_id)?;
            if let DeviceRefEnum::ModbusBus(master_device) = master_device_enum.borrow() {
                // make do controller device
                let do_controller = do_controller_factory::make(
                    device_json, 
                    master_device
                )?;
                Ok(DeviceRefEnum::ModbusDoController(Rc::new(do_controller)))
            } else {
                Err(DriverError(format!(
                    "device factory: cannot find master device: {}",
                    master_device_id
                )))
            }

        } else if device_type == "modbus_do_port" {
            let master_device_id = device_json["master_device_id"].as_str().ok_or(
                DriverError("device factory: no master_device_id in config".to_string())
            )?;
            // get modbus master device
            let master_device_enum = self.get_master_device_enum(master_device_id)?;
            if let DeviceRefEnum::ModbusDoController(master_device) = master_device_enum.borrow() {
                // make do port device
                let do_port = do_port_factory::make(
                    device_json, 
                    master_device
                )?;
                Ok(DeviceRefEnum::ModbusDoPort(Rc::new(do_port)))
            } else {
                Err(DriverError(format!(
                    "device factory: cannot find master device: {}",
                    master_device_id
                )))
            }
        } else {
            Err(DriverError(format!(
                "device factory: unknown device type: {}",
                device_type
            )))
        }
    }

    fn get_master_device_enum(&self, device_id: &str) -> Result<&DeviceRefEnum, DriverError> {
        let master_device_enum = self
           .device_map
           .get(device_id)
           .ok_or(DriverError(format!(
                "device factory: cannot find master device: {}",
                device_id
            )))?;
        Ok(master_device_enum)
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
