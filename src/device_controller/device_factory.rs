//! device factory registery

use super::entity::device_enum::DeviceRefEnum;
use super::entity::device_po::DevicePo;
use super::factory::*;
use crate::driver::modbus::traits::{ModbusDiControllerListener, ModbusListener};
use crate::driver::modbus::{modbus_bus, modbus_di_controller_coil};
use crate::entity::dto::device_meta_info_dto::{DeviceMetaInfoDto, DeviceStatusEnum};
use crate::entity::dto::device_state_dto::StateToDeviceControllerDto;
use crate::{common::error::DriverError, driver::modbus::modbus_bus::ModbusBus};
use crate::{debug, error, info, trace, warn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};

const LOG_TAG: &str = "device_factory";

/// the factory for making devices
/// you will first call "read_json()" to read data from json Value
/// then, call "make_device_by_config_map()" to init all device object
/// last, call "get_result()" to get device info map and device enum map, after that, DeviceFactory will drop
pub struct DeviceInstanceFactory {
    device_enum_map: HashMap<String, DeviceRefEnum>,
    device_info_map: Arc<Mutex<HashMap<String, DeviceMetaInfoDto>>>,
    device_po_list: Vec<DevicePo>,
    report_tx_dummy: mpsc::Sender<StateToDeviceControllerDto>,
}

impl DeviceInstanceFactory {
    pub fn new(report_tx_dummy: mpsc::Sender<StateToDeviceControllerDto>) -> DeviceInstanceFactory {
        DeviceInstanceFactory {
            device_enum_map: HashMap::new(),
            device_info_map: Arc::new(Mutex::new(HashMap::new())),
            device_po_list: Vec::new(),
            report_tx_dummy,
        }
    }

    /// get results after making all devices
    /// return all maps to device manager
    /// after calling this function, this DeviceFactory will drop
    pub fn get_device_map(self) -> HashMap<String, DeviceRefEnum> {
        self.device_enum_map
    }

    pub fn make_devices(
        &mut self,
        device_info_map: Arc<Mutex<HashMap<String, DeviceMetaInfoDto>>>,
        device_po_list: Vec<DevicePo>,
    ) -> Result<(), DriverError> {
        // set device infomartion
        self.device_po_list = device_po_list;
        self.device_info_map = device_info_map;

        // clone template information
        let device_po_list = self.device_po_list.clone();
        // used for making device object
        // copy device info map for making device

        let device_info_map: HashMap<String, DeviceMetaInfoDto>;

        {
            let device_map_guard = self
                .device_info_map
                .lock()
                .map_err(|e| DriverError(format!("get device info map mutex error: {}", e)))?;
            device_info_map = device_map_guard.clone();
        }

        for device_po in device_po_list {
            let device_info =
                device_info_map
                    .get(device_po.device_id.as_str())
                    .ok_or(DriverError(format!(
                        "cannot find device info for device_id: {}",
                        device_po.device_id.as_str()
                    )))?;
            // 1. use device info to make device object, create_device will put device object to device_enum_map, and will change DeviceInfoDto statue to initialized
            let result = self.create_device(&device_info);

            // 2. update device status to "Initialized"
            {
                let mut device_map_guard = self
                    .device_info_map
                    .lock()
                    .map_err(|e| DriverError(format!("get device info map mutex error: {}", e)))?;
                if let Some(data_mut) = device_map_guard.get_mut(device_po.device_id.as_str()) {
                    data_mut.device_status = DeviceStatusEnum::Initialized;
                } else {
                    return Err(DriverError(format!(
                        "update device status failed, getting mut reference failed, device_id: {}",
                        device_po.device_id
                    )));
                }
            }

            match result {
                Ok(_) => {}
                Err(e) => {
                    error!(
                        LOG_TAG,
                        "create device failed, device_id: {}, error msg: {}",
                        device_po.device_id,
                        e.0.as_str()
                    );
                }
            }
        }
        Ok(())
    }

    /// make device by one device info bo
    /// this function will make the device and change device_enum map
    fn create_device(&mut self, dto: &DeviceMetaInfoDto) -> Result<(), DriverError> {
        if dto.device_type == "modbus_bus" {
            let _ = self.make_modbus(dto)?;
        } else if dto.device_type == "serial_bus" {
            let _ = self.make_serial_bus(dto)?;
        } else if dto.device_type == "dmx_bus" {
            let _ = self.make_dmx_bus(dto)?;
        } else if dto.device_type == "modbus_do_controller" {
            let _ = self.make_do_controller(dto)?;
        } else if dto.device_type == "modbus_do_port" {
            let _ = self.make_do_port(dto)?;
        } else if dto.device_type == "modbus_di_controller" {
            let _ = self.make_di_controller(dto)?;
        } else if dto.device_type == "modbus_di_port" {
            let _ = self.make_di_port(dto)?;
        } else if dto.device_type == "remote" {
            let _ = self.make_remote_controller(dto)?;
        } else if dto.device_type == "audio" {
            let _ = self.make_audio(dto)?;
        } else if dto.device_type == "dmx_channel" {
            let _ = self.make_dmx_channel_device(dto)?;
        } else {
            return Err(DriverError(format!(
                "unknown device type. device_type={}",
                dto.device_type
            )));
        }
        info!(
            LOG_TAG,
            "device created! device_id: {}, device_type: {}", dto.device_id, dto.device_type
        );
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

    fn make_audio(&mut self, dto: &DeviceMetaInfoDto) -> Result<(), DriverError> {
        let audio = audio_factory::make(&dto, self.report_tx_dummy.clone())?;
        self.device_enum_map.insert(
            dto.device_id.clone(),
            DeviceRefEnum::Audio(Rc::new(RefCell::new(audio))),
        );
        Ok(())
    }

    fn make_modbus(&mut self, dto: &DeviceMetaInfoDto) -> Result<(), DriverError> {
        let modbus_bus = modbus_bus_factory::make(&dto, self.report_tx_dummy.clone())?;
        self.device_enum_map.insert(
            dto.device_id.clone(),
            DeviceRefEnum::ModbusBus(Rc::new(RefCell::new(modbus_bus))),
        );
        Ok(())
    }

    fn make_dmx_bus(&mut self, bo: &DeviceMetaInfoDto) -> Result<(), DriverError> {
        let dmx_bus = dmx_bus_factory::make(&bo, self.report_tx_dummy.clone())?;
        self.device_enum_map.insert(
            bo.device_id.clone(),
            DeviceRefEnum::DmxBus(Rc::new(RefCell::new(dmx_bus))),
        );
        Ok(())
    }

    fn make_dmx_channel_device(&mut self, dto: &DeviceMetaInfoDto) -> Result<(), DriverError> {
        if let Some(master_device_id) = &dto.master_device_id {
            // 1 get dmx bus master device
            let master_device_enum = self.get_master_device_enum(master_device_id.as_str())?;

            if let DeviceRefEnum::DmxBus(dmx_bus) = master_device_enum {
                // 2 make dmx channel
                let dmx_channel = channel_device_factory::make(
                    &dto,
                    dmx_bus.clone(),
                    self.report_tx_dummy.clone(),
                )?;
                self.device_enum_map.insert(
                    dto.device_id.clone(),
                    DeviceRefEnum::DmxChannel(Rc::new(RefCell::new(dmx_channel))),
                );
                Ok(())
            } else {
                Err(DriverError(format!(
                    "device factory: cannot find master device during dmx channel init, master_device_id={}, device_id={}",
                    master_device_id,
                    dto.device_id
                )))
            }
        } else {
            Err(DriverError(format!(
                "device_factory: no master_device_id for dmx channel, device_id={}",
                dto.device_id
            )))
        }
    }

    fn make_serial_bus(&mut self, dto: &DeviceMetaInfoDto) -> Result<(), DriverError> {
        let serial_bus = serial_bus_factory::make(&dto)?;
        self.device_enum_map.insert(
            dto.device_id.clone(),
            DeviceRefEnum::SerialBus(Rc::new(RefCell::new(serial_bus))),
        );
        Ok(())
    }

    fn make_remote_controller(&mut self, dto: &DeviceMetaInfoDto) -> Result<(), DriverError> {
        if let Some(master_device_id) = &dto.master_device_id {
            // get serial master device
            let master_device_enum = self.get_master_device_enum(&master_device_id.as_str())?;
            if let DeviceRefEnum::SerialBus(serial_bus_device) = master_device_enum {
                // make remote controller
                let remote_controller = remote_factory::make(&dto, self.report_tx_dummy.clone())?;
                let mut serial_bus = serial_bus_device.borrow_mut();
                serial_bus.add_listener(Box::new(remote_controller));
                Ok(())
            } else {
                Err(DriverError(format!(
                    "device facotry: cannot make serial remote controller, master_device_id={}, device_id={}",
                    master_device_id,
                    dto.device_id
                )))
            }
        } else {
            Err(DriverError(format!(
                "device factory: no master_device_id for do controller, device_id={}",
                dto.device_id
            )))
        }
    }

    fn make_do_controller(&mut self, dto: &DeviceMetaInfoDto) -> Result<(), DriverError> {
        if let Some(master_device_id) = &dto.master_device_id {
            // get modbus master device
            let master_device_enum = self.get_master_device_enum(master_device_id.as_str())?;
            if let DeviceRefEnum::ModbusBus(master_device) = master_device_enum {
                // make do controller device
                let do_controller =
                    do_controller_factory::make(&dto, master_device, self.report_tx_dummy.clone())?;
                self.device_enum_map.insert(
                    dto.device_id.clone(),
                    DeviceRefEnum::ModbusDoController(Rc::new(RefCell::new(do_controller))),
                );
                Ok(())
            } else {
                Err(DriverError(format!(
                    "device factory: cannot find master device during do controller init, master_device_id={}, device_id={}",
                    master_device_id,
                    dto.device_id
                )))
            }
        } else {
            Err(DriverError(format!(
                "device_factory: no master_device_id for do controller, device_id={}",
                dto.device_id
            )))
        }
    }

    fn make_do_port(&mut self, dto: &DeviceMetaInfoDto) -> Result<(), DriverError> {
        if let Some(master_device_id) = &dto.master_device_id {
            // get modbus master device
            let master_device_enum = self.get_master_device_enum(master_device_id.as_str())?;
            if let DeviceRefEnum::ModbusDoController(master_device) = master_device_enum {
                // make do port device
                let do_port = do_port_factory::make(
                    &dto,
                    Rc::clone(master_device),
                    self.report_tx_dummy.clone(),
                )?;
                let _ = self.device_enum_map.insert(
                    dto.device_id.clone(),
                    DeviceRefEnum::ModbusDoPort(Rc::new(RefCell::new(do_port))),
                );
                Ok(())
            } else {
                Err(DriverError(format!(
                    "device factory: the master device is not modbus_controller during making do port: master_device_id={}, device_id={}",
                    master_device_id,
                    dto.device_id
                )))
            }
        } else {
            Err(DriverError(format!(
                "device_factory: do not find master device_id for do port, device_id={}",
                dto.device_id
            )))
        }
    }

    fn make_di_controller(&mut self, dto: &DeviceMetaInfoDto) -> Result<(), DriverError> {
        // init modbus_di_controller
        // get master device (modbus), and insert itself into it
        if let Some(master_device_id) = &dto.master_device_id {
            // get modbus master device
            let master_device_enum = self.get_master_device_enum(master_device_id.as_str())?;
            if let DeviceRefEnum::ModbusBus(master_modbus_ref) = master_device_enum {
                // make di controller device
                let di_controller =
                    di_controller_factory::make(&dto, self.report_tx_dummy.clone())?;
                // mount to modbus bus device
                let mut modbus = master_modbus_ref.borrow_mut();
                modbus.add_di_controller(di_controller.get_unit(), Box::new(di_controller));
                Ok(())
            } else {
                Err(DriverError(format!(
                    "device factory: when init di_controller, the master device is not modbus_bus, master_device_id: {}, device_id: {}",
                    master_device_id,
                    dto.device_id
                )))
            }
        } else {
            Err(DriverError(format!(
                "device_factory: do not find master_device_id for di port, device_id={}",
                dto.device_id
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
    fn make_di_port(&mut self, dto: &DeviceMetaInfoDto) -> Result<(), DriverError> {
        // find modbus_di_controller, and insert modbus_di_port into it
        if let Some(master_device_id) = &dto.master_device_id {
            // find modbus controller's master_device_id
            let controller_master_device_id: String;

            {
                let device_map_guard = self
                    .device_info_map
                    .lock()
                    .map_err(|e| DriverError(format!("get device info map mutex error: {}", e)))?;
                controller_master_device_id = device_map_guard.get(master_device_id.as_str())
                            .ok_or(DriverError(format!(
                                "device factory: cannot find master_device_id of di_controller for di port, di_controller_id: {}, device_id: {}",
                                master_device_id, dto.device_id
                            )))?.master_device_id.clone().ok_or(
                                DriverError(format!(
                                    "device factory: cannot find master_device_id of di_controller for di port, di_controller_id: {}, device_id: {}",
                                    master_device_id, dto.device_id
                                ))
                            )?;
            }

            // borrow modbus bus
            let master_device_enum =
                self.get_master_device_enum(controller_master_device_id.as_str())?;
            if let DeviceRefEnum::ModbusBus(master_modbus_ref) = master_device_enum {
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
                    let di_port = di_port_factory::make(&dto, self.report_tx_dummy.clone())?;
                    // mount to modbus_di_controller
                    let mut di_controller = master_di_controller_ref.borrow_mut();
                    di_controller.add_di_port(di_port.get_address(), Box::new(di_port))?;
                    Ok(())
                } else {
                    return Err(DriverError(format!(
                                "device factory: no master_di_controller for modbus_di_port in device config: master_device_id: {}, device_id: {}",
                                master_device_id,
                                dto.device_id
                            )));
                }
            } else {
                Err(DriverError(format!(
                            "device factory: no find master device for modbus_di_port, master_device_id: {}, device_id: {}",
                            master_device_id,
                            dto.device_id
                        )))
            }
        } else {
            Err(DriverError(format!(
                "device_factory: no master_device_id for di port, device_id={}",
                dto.device_id
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
