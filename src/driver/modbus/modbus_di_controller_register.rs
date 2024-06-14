use std::sync::mpsc::Sender;
use std::{collections::HashMap, hash::Hash};
use crate::common::error::DriverError;
use crate::driver::traits::ReportUpward;
use crate::entity::dto::device_report_dto::DeviceReportDto;
use crate::entity::dto::device_state_dto::{StateToDeviceControllerDto, DiControllerStateDto, StateDtoEnum};
use super::traits::{ModbusControllerType, ModbusDiControllerListener, ModbusListener};
use super::prelude::*;
use crate::{info, warn, error, trace, debug};

const LOG_TAG: &str = "modbus_di_controller";
const DEVICE_CLASS: &str = "operable";
const DEVICE_TYPE: &str = "modbus_di_controller";

/// Modbus Digital Input Controller
/// - Cache data on the controller
/// - read data from modbus and relay to selector port object
pub struct ModbusDiControllerRegsiter {
    device_id: String,
    unit: ModbusUnitSize,
    // modbus input port number
    input_num: ModbusAddrSize, 
    // modbus controller port object map
    mount_port_map:  HashMap<ModbusAddrSize, Box<dyn ModbusDiControllerListener + Send>>,
    // port state cache
    port_state_vec: Vec<bool>,
    report_tx: Sender<StateToDeviceControllerDto>,
    error_msg: Option<String>,
    error_timestamp: Option<u64>,
    last_update: Option<u64>,
}

impl ReportUpward for ModbusDiControllerRegsiter {
    fn get_upward_channel(&self) -> &Sender<StateToDeviceControllerDto> {
        return &self.report_tx;
    }

    fn report(&self) -> Result<(), DriverError> {
        let state_dto = DiControllerStateDto {
            port: self.port_state_vec.clone()
        };
        self.notify_upward(StateToDeviceControllerDto {
            device_id: self.device_id.clone(),
            device_class: DEVICE_CLASS.to_string(),
            device_type: DEVICE_TYPE.to_string(),
            status: DeviceReportDto{
                active: true,
                error_msg: self.error_msg.clone(),
                error_timestamp: self.error_timestamp.clone(),
                last_update: self.last_update.clone(),
                state: StateDtoEnum::DiController(state_dto.clone())
            }
        })?;
        Ok(())
    }
}

impl ModbusListener for ModbusDiControllerRegsiter {
    fn get_controller_type(&self) -> ModbusControllerType {
        ModbusControllerType::Register
    }

    fn get_unit(&self) -> ModbusUnitSize {
        self.unit
    }

    fn get_port_num(&self) -> ModbusAddrSize {
        self.input_num
    }

    /// mount port object 
    fn add_di_port(&mut self, address: ModbusAddrSize, port_to_mount: Box<dyn ModbusDiControllerListener + Send>) -> Result<(), DriverError> {
        self.mount_port_map.insert(address, port_to_mount);
        info!(LOG_TAG, "port mounted, device_id: {} address: {}", &self.device_id, &address);
        Ok(())
    }

    /// read data from modbus and relay to port object
    /// - if data not change, do nothing
    /// - if data changed, notify port object
    /// - TODO 优化：可将缓存的数据和传入的数据保存为按位的整型，然后按位比较，可更快找到差异位置，然后通知下游
    fn notify_from_bus(&mut self, address: ModbusAddrSize, messages: Vec<bool>) -> Result<(), DriverError> {

        debug!(LOG_TAG, "received from modbus, address: {}, messages: {:?}", &address, &messages);

        // check if there is device exist, if there is not, ignore
        if !self.mount_port_map.contains_key(&address) {
            debug!(LOG_TAG, "cannot find mount port, address: {}", &address);
            return Ok(())
        }

        // check if data changed
        let port = self.mount_port_map.get(&address).ok_or(DriverError("DiController 接收到的消息，没有找到对应的端口".to_string()))?;
        for (i, message) in messages.iter().enumerate() {
            if self.port_state_vec[i] != *message {
                // if data changed, notify port object
                self.notify_port(address, *message)?;
                debug!(LOG_TAG, "port status changed, address: {}, messages: {:?}", &address, &messages);
            }
        }
        self.port_state_vec = messages.clone();
        self.report()?;
        Ok(())
    }

    /// notify modbus port
    fn notify_port(&self, address: ModbusAddrSize, message: bool) -> Result<(), DriverError> {
        // check if the port exists
        if !self.mount_port_map.contains_key(&address) {
            return Ok(())
        }

        // send message to port
        let port: &Box<dyn ModbusDiControllerListener + Send> = self.mount_port_map.get(&address).ok_or(DriverError("DiController 向端口发送消息失败，没有找到对应的端口".to_string()))?;
        port.notify(message)?;
        Ok(())
    }
}

impl ModbusDiControllerRegsiter {
    pub fn new(device_id: &str, unit: ModbusUnitSize, input_num: ModbusAddrSize, report_tx: Sender<StateToDeviceControllerDto>) -> Self {
        Self {
            device_id: device_id.to_string(),
            unit,
            input_num,
            mount_port_map: HashMap::new(),
            port_state_vec: vec![false; input_num as usize],
            report_tx,
            error_msg: None,
            error_timestamp: None,
            last_update: None,
        }
    }

    pub fn get_unit(&self) -> ModbusUnitSize {
        self.unit
    }
}

#[cfg(test)]
mod tests {}