//! device factory registery

use super::entity::device_enum::DeviceRefEnum;
use super::entity::device_info::DeviceInfoBo;
use super::factory::*;
use crate::driver::dmx::dmx_bus::DmxBus;
use crate::driver::modbus::{modbus_bus, modbus_di_controller};
use crate::driver::serial::serial_bus::SerialBus;
use crate::entity::bo::device_state_bo::DeviceStateBo;
use crate::util::json::get_str;
use crate::{common::error::DriverError, driver::modbus::modbus_bus::ModbusBus};
use crate::{debug, error, info, trace, warn};
use lazy_static::lazy_static;
use serde_json::Value;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use crate::driver::modbus::traits::{ModbusDiControllerListener, ModbusListener};

const LOG_TAG: &str = "device_factory";

struct DeviceFactory {
    device_enum_map: HashMap<String, DeviceRefEnum>,
    device_info_map: HashMap<String, DeviceInfoBo>,
    upward_rx_dummy: mpsc::Sender<DeviceStateBo>,
}

impl DeviceFactory {
    fn new(upward_rx_dummy: mpsc::Sender<DeviceStateBo>) -> DeviceFactory {
        DeviceFactory {
            device_enum_map: HashMap::new(),
            device_info_map: HashMap::new(),
            upward_rx_dummy
        }
    }

    /// read json and load into device_info_map
    fn load_json(&mut self, config_list: Value) -> Result<(), DriverError> {
        // device map area
        let device_list = config_list.as_array().ok_or(DriverError(
            "device factory: cannot find modbus_bus in config".to_string(),
        ))?;

        let mut config_list = device_list.clone();

        for device_config in config_list.iter() {
            let device_id = device_config["device_id"].as_str().ok_or(DriverError(
                "device factory: cannot find device_id in config".to_string(),
            ))?;

            let device_type = device_config["device_type"].as_str().ok_or(DriverError(
                "device factory: no device_type in config".to_string(),
            ))?;

            let master_device_id = device_config["master_device_id"]
                .as_str()
                .map(|s| s.to_string());

            let device_info = DeviceInfoBo {
                device_id: device_id.to_string(),
                device_type: device_type.to_string(),
                master_device_id: master_device_id,
                config: device_config.clone(),
            };

            self.device_info_map
                .insert(device_id.to_string(), device_info);
        }

        Ok(())
    }

    /// make device with config map
    fn make_device_with_config_map(&mut self, config_list: Value) -> Result<(), DriverError> {
        for device_config in config_list.iter() {
            let device_id = device_config["device_id"].as_str().ok_or(DriverError(
                "device factory: cannot find device_id in config".to_string(),
            ))?;

            if self.device_enum_map.contains_key(device_id) {
                continue;
            }

            let device = self.make_device(device_config)?;
            self.device_enum_map.insert(device_id.to_string(), device);
        }

        Ok(())
    }

    fn make_device(&mut self, device_id: &str) -> Result<Option<DeviceRefEnum>, DriverError> {
        if let Some(bo) = self.device_info_map.get(device_id) {
            if bo.device_type == "modbus_bus" {
                let modbus_bus = modbus_bus_factory::make(&bo.config)?;
                Ok(Some(DeviceRefEnum::ModbusBus(Rc::new(RefCell::new(modbus_bus)))))

            } else if bo.device_type == "serial_bus" {
                let serial_bus = serial_bus_factory::make(&bo.config)?;
                Ok(Some(DeviceRefEnum::SerialBus(Rc::new(RefCell::new(serial_bus)))))

            } else if bo.device_type == "dmx_bus" {
                let dmx_bus = dmx_bus_factory::make(&bo.config)?;
                Ok(Some(DeviceRefEnum::DmxBus(Rc::new(RefCell::new(dmx_bus)))))

            } else if bo.device_type == "modbus_do_controller" {
                if let Some(master_device_id) = bo.master_device_id {
                    // get modbus master device
                    let master_device_enum = self.get_master_device_enum(master_device_id.as_str())?;
                    if let DeviceRefEnum::ModbusBus(master_device) = master_device_enum.borrow() {
                        // make do controller device
                        let do_controller = do_controller_factory::make(&bo.config, master_device)?;
                        Ok(Some(DeviceRefEnum::ModbusDoController(Rc::new(RefCell::new(do_controller)))))
                    } else {
                        Err(DriverError(format!(
                            "device factory: cannot find master device: {}",
                            master_device_id
                        )))
                    }
                } else {
                    Err(DriverError(format!(
                        "device_factory: do not find master device_id for do controller"
                    )))
                }

            } else if bo.device_type == "modbus_do_port" {
                if let Some(master_device_id) = bo.master_device_id {
                    // get modbus master device
                    let master_device_enum = self.get_master_device_enum(master_device_id.as_str())?;
                    if let DeviceRefEnum::ModbusDoController(master_device) =
                        master_device_enum.borrow()
                    {
                        // make do port device
                        let do_port = do_port_factory::make(&bo.config, Rc::clone(master_device))?;
                        Ok(Some(DeviceRefEnum::ModbusDoPort(Rc::new(RefCell::new(do_port)))))
                    } else {
                        Err(DriverError(format!(
                            "device factory: cannot find master device: {}",
                            master_device_id
                        )))
                    }
                } else {
                    Err(DriverError(format!(
                        "device_factory: do not find master device_id for do port"
                    )))
                }

            } else if bo.device_type == "modbus_di_controller" {
                // init modbus_di_controller
                // get master device (modbus), and insert itself into it 
                if let Some(master_device_id) = bo.master_device_id {
                    // get modbus master device
                    let master_device_enum = self.get_master_device_enum(master_device_id.as_str())?;
                    if let DeviceRefEnum::ModbusBus(master_modbus_ref) =
                        master_device_enum.borrow()
                    {
                        // make di controller device
                        let di_controller = di_controller_factory::make(&bo.config)?;
                        // mount to modbus bus device
                        if let modbus = master_modbus_ref.borrow_mut() {
                            modbus.add_di_controller(di_controller.get_unit(), Box::new(di_controller));
                            Ok(None)
                        } else {
                            Err(DriverError(format!(
                                "device factory: the master device for modbus_di_controller is not modbus bus: {}",
                                master_device_id
                            )))
                        }
                    } else {
                        Err(DriverError(format!(
                            "device factory: cannot find master device for modbus_di_controller: {}",
                            master_device_id
                        )))
                    }
                } else {
                    Err(DriverError(format!(
                        "device_factory: do not find master device_id for di port"
                    )))
                }

            } else if bo.device_type == "modbus_di_port" {
                // make modbus_di_port
                // get master deivce (modbus_do_controller), and insert modbus_do_port into it
                if let Some(master_device_id) = bo.master_device_id {
                    // get modbus master device
                    let master_device_enum = self.get_master_device_enum(master_device_id.as_str())?;
                    if let DeviceRefEnum::ModbusDiController(master_modbus_ref) =
                        master_device_enum.borrow()
                    {
                        // make di port device
                        let di_port = di_port_factory::make(&bo.config, self.upward_rx_dummy.clone())?;
                        // mount to modbus di controller device
                        if let di_controller = master_modbus_ref.borrow_mut() {
                            di_controller.add_di_port(di_port.get_unit(), Box::new(di_port));
                            Ok(None)
                        } else {
                            Err(DriverError(format!(
                                "device factory: the master device for modbus_di_port is not modbus_di_controller: {}",
                                master_device_id
                            )))
                        }
                    } else {
                        Err(DriverError(format!(
                            "device factory: cannot find master device for modbus_di_port: {}",
                            master_device_id
                        )))
                    }

                } else {
                    Err(DriverError(format!(
                        "device_factory: do not find master device_id for di port"
                    )))
                }

            } else {
                Err(DriverError(format!(
                    "device factory: unknown device type: {}",
                    bo.device_type
                )))
            }
        } else {
            Err(DriverError(format!(
                "device factory: cannot find device info: {}",
                device_id
            )))
        }
    }

    fn get_master_device_enum(&self, device_id: &str) -> Result<&DeviceRefEnum, DriverError> {
        let master_device_enum =
            self.device_enum_map
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
