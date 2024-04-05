//! device factory registery

use super::entity::device_enum::DeviceRefEnum;
use super::entity::device_info::{DeviceInfoBo, DeviceStatusEnum};
use super::factory::*;
use crate::driver::dmx::dmx_bus::DmxBus;
use crate::driver::modbus::traits::{ModbusDiControllerListener, ModbusListener};
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

const LOG_TAG: &str = "device_factory";

/// the factory for making devices
/// you will first call "read_json()" to read data from json Value
/// then, call "make_device_by_config_map()" to init all device object
/// last, call "get_result()" to get device info map and device enum map, after that, DeviceFactory will drop
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
            upward_rx_dummy,
        }
    }
    
    /// get results after making all devices
    /// return all maps to device manager
    /// after calling this function, this DeviceFactory will drop
    pub fn get_result(self) -> (HashMap<String, DeviceInfoBo>, HashMap<String, DeviceRefEnum>) {
        ( self.device_info_map, self.device_enum_map )
    }

    /// read json and load into device_info_map
    pub fn read_json(&mut self, config_list: Value) -> Result<(), DriverError> {
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
                status: DeviceStatusEnum::NotInitialized
            };

            self.device_info_map
                .insert(device_id.to_string(), device_info);
        }

        Ok(())
    }

    /// make device with config map
    pub fn make_device_by_config_map(&mut self) -> Result<(), DriverError> {
        let device_info_map_copy = self.device_info_map.clone();
        for device_config in device_info_map_copy.iter() {
            let device_id = device_config.0; 

            // regressive checking that the device is already initialized
            if let Some(device_config) = device_info_map_copy.get(device_id) {
                if device_config.status == DeviceStatusEnum::Initialized {
                    continue;
                }
            }

            let _ = self.create_device(device_config.1)?;
        }

        Ok(())
    }

    /// make device by one device info bo
    /// this function will make the device and change device_enum map
    fn create_device(&mut self, bo: &DeviceInfoBo) -> Result<(), DriverError> {
        if bo.device_type == "modbus_bus" {
            self.make_modbus(bo)?
        } else if bo.device_type == "serial_bus" {
            self.make_serial_bus(bo)?
        } else if bo.device_type == "dmx_bus" {
            self.make_dmx_bus(bo)?
        } else if bo.device_type == "modbus_do_controller" {
            self.make_do_controller(bo)?
        } else if bo.device_type == "modbus_do_port" {
            self.make_do_port(bo)?;
        } else if bo.device_type == "modbus_di_controller" {
            let _ = self.make_di_controller(bo)?;
        } else if bo.device_type == "modbus_di_port" {
            let _ = self.make_di_port(bo)?;
        } else {
            return Err(DriverError(format!(
                "device factory: unknown device type. device_type={}, device_id={}",
                bo.device_type,
                bo.device_id
            )))
        }
        if let Some(data_mut) = self.device_info_map.get_mut(bo.device_id.as_str()) {
            data_mut.status = DeviceStatusEnum::Initialized;
        } else {
            return Err(DriverError(format!(
                "device factory: update device status failed, mut reference getting failed, device_id: {}",
                bo.device_id
            )))
        }
        Ok(())
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

    fn make_modbus(&mut self, bo: &DeviceInfoBo) -> Result<(), DriverError> {
        let modbus_bus = modbus_bus_factory::make(&bo.config)?;
        self.device_enum_map.insert(bo.device_id.clone(), DeviceRefEnum::ModbusBus(Rc::new(RefCell::new(modbus_bus))));
        Ok(())
    }

    fn make_dmx_bus(&mut self, bo: &DeviceInfoBo) -> Result<(), DriverError> {
        let dmx_bus = dmx_bus_factory::make(&bo.config)?;
        self.device_enum_map.insert(bo.device_id.clone(), DeviceRefEnum::DmxBus(Rc::new(RefCell::new(dmx_bus))));
        Ok(())
    }

    fn make_serial_bus(&mut self, bo: &DeviceInfoBo) -> Result<(), DriverError> {
        let serial_bus = serial_bus_factory::make(&bo.config)?;
        self.device_enum_map.insert(bo.device_id.clone(), DeviceRefEnum::SerialBus(Rc::new(RefCell::new(serial_bus))));
        Ok(())
    }

    fn make_do_controller(&mut self, bo: &DeviceInfoBo) -> Result<(), DriverError> {
        if let Some(master_device_id) = &bo.master_device_id {
            // get modbus master device
            let master_device_enum = self.get_master_device_enum(master_device_id.as_str())?;
            if let DeviceRefEnum::ModbusBus(master_device) = master_device_enum.borrow() {
                // make do controller device
                let do_controller = do_controller_factory::make(&bo.config, master_device)?;
                self.device_enum_map.insert(bo.device_id.clone(), DeviceRefEnum::ModbusDoController(Rc::new(RefCell::new(
                    do_controller,
                ))));
                Ok(())
            } else {
                Err(DriverError(format!(
                    "device factory: cannot find master device during do controller init, master_device_id={}, device_id={}",
                    master_device_id,
                    bo.device_id
                )))
            }
        } else {
            Err(DriverError(format!(
                "device_factory: no master_device_id for do controller, device_id={}",
                bo.device_id
            )))
        }
    }

    fn make_do_port(&mut self, bo: &DeviceInfoBo) -> Result<(), DriverError> {
        if let Some(master_device_id) = &bo.master_device_id {
            // get modbus master device
            let master_device_enum = self.get_master_device_enum(master_device_id.as_str())?;
            if let DeviceRefEnum::ModbusDoController(master_device) = master_device_enum.borrow() {
                // make do port device
                let do_port = do_port_factory::make(&bo.config, Rc::clone(master_device))?;
                let _ = self.device_enum_map.insert(bo.device_id.clone(), DeviceRefEnum::ModbusDoPort(Rc::new(RefCell::new(do_port))));
                Ok(())
            } else {
                Err(DriverError(format!(
                    "device factory: the master device is not modbus_controller during making do port: master_device_id={}, device_id={}",
                    master_device_id,
                    bo.device_id
                )))
            }
        } else {
            Err(DriverError(format!(
                "device_factory: do not find master device_id for do port, device_id={}",
                bo.device_id
            )))
        }
    }

    fn make_di_controller(&mut self, bo: &DeviceInfoBo) -> Result<(), DriverError> {
        // init modbus_di_controller
        // get master device (modbus), and insert itself into it
        if let Some(master_device_id) = &bo.master_device_id {
            // get modbus master device
            let master_device_enum = self.get_master_device_enum(master_device_id.as_str())?;
            if let DeviceRefEnum::ModbusBus(master_modbus_ref) = master_device_enum.borrow() {
                // make di controller device
                let di_controller = di_controller_factory::make(&bo.config)?;
                // mount to modbus bus device
                if let mut modbus = master_modbus_ref.borrow_mut() {
                    modbus.add_di_controller(di_controller.get_unit(), Box::new(di_controller));
                    Ok(())
                } else {
                    Err(DriverError(format!(
                        "device factory: caonnot borrow modbus, master_device_id={}, device_id={}",
                        master_device_id, bo.device_id
                    )))
                }
            } else {
                Err(DriverError(format!(
                    "device factory: when init di_controller, the master device is not modbus_bus, master_device_id: {}, device_id: {}",
                    master_device_id,
                    bo.device_id
                )))
            }
        } else {
            Err(DriverError(format!(
                "device_factory: do not find master_device_id for di port, device_id={}",
                bo.device_id
            )))
        }
    }

    /// make di port is complicated, because di_port was hold by di_controller, and the controller was hold by modbus, the steps are:
    /// 1 find master_device_id (di_controller)
    /// 2 find di_controller's master deivce (modbus)
    /// 3 bowrrow modbus
    /// 4 borrow di_controller
    /// 5 make di_port device
    /// 6 mount di_port onto di_controller
    fn make_di_port(&mut self, bo: &DeviceInfoBo) -> Result<(), DriverError> {
        // find modbus_di_controller, and insert modbus_di_port into it
        if let Some(master_device_id) = &bo.master_device_id {
            // find modbus controller's master_device_id
            let controller_master_device_id = self.device_info_map.get(master_device_id.as_str())
                        .ok_or(DriverError(format!(
                            "device factory: cannot find master_device_id of di_controller for di port, di_controller_id: {}, device_id: {}",
                            master_device_id, bo.device_id
                        )))?.master_device_id.clone().ok_or(
                            DriverError(format!(
                                "device factory: cannot find master_device_id of di_controller for di port, di_controller_id: {}, device_id: {}",
                                master_device_id, bo.device_id
                            ))
                        )?;
            // borrow modbus bus
            let master_device_enum =
                self.get_master_device_enum(controller_master_device_id.as_str())?;
            if let DeviceRefEnum::ModbusBus(master_modbus_ref) = master_device_enum.borrow() {
                // borrow modbus_di_controller
                if let DeviceRefEnum::ModbusDiController(master_di_controller_ref) = self
                    .device_enum_map
                    .get(master_device_id.as_str())
                    .ok_or(DriverError(format!(
                        "device factory: cannot find master device for modbus_di_port: {}",
                        master_device_id
                    )))?
                {
                    // make di port device
                    let di_port = di_port_factory::make(&bo.config, self.upward_rx_dummy.clone())?;
                    // mount to modbus_di_controller
                    if let mut di_controller = master_di_controller_ref.borrow_mut() {
                        di_controller.add_di_port(di_port.get_address(), Box::new(di_port));
                        Ok(())
                    } else {
                        Err(DriverError(format!(
                                    "device factory: wrong master device for modbus_di_port is not modbus_di_controller: {}, device_id: {}",
                                    master_device_id,
                                    bo.device_id
                                )))
                    }
                } else {
                    return Err(DriverError(format!(
                                "device factory: no master_di_controller for modbus_di_port in device config: master_device_id: {}, device_id: {}",
                                master_device_id,
                                bo.device_id
                            )));
                }
            } else {
                Err(DriverError(format!(
                            "device factory: no find master device for modbus_di_port, master_device_id: {}, device_id: {}",
                            master_device_id,
                            bo.device_id
                        )))
            }
        } else {
            Err(DriverError(format!(
                "device_factory: no master_device_id for di port, device_id={}",
                bo.device_id
            )))
        }
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
        // let modbus_config_value = json!([{
        //     "device_id": "modbus_bus_test_1",
        //     "device_type": "modbus_bus",
        //     "config": {
        //         "serial_port": "/dev/ttyUSB0",
        //         "baudrate": 115200,
        //     }
        // }]);
        // let mut device_factory = DeviceFactory::new();
        // let _ = device_factory.make_device_with_config_map(modbus_config_value);
        // println!("done");
    }
}
