use std::{collections::HashMap, hash::Hash};
use crate::common::error::DriverError;
use super::traits::{ModbusListener, ModbusDiControllerListener};
use super::prelude::*;
use crate::{info, warn, error, trace, debug};

const LOG_TAG: &str = "Modbus Di Controller";

/// Modbus Digital Input Controller
/// - Cache data on the controller
/// - read data from modbus and relay to selector port object
pub struct ModbusDiController {
    device_id: String,
    unit: ModbusUnitSize,
    // modbus input port number
    input_num: ModbusAddrSize, 
    // modbus controller port object map
    mount_port_map:  HashMap<ModbusAddrSize, Box<dyn ModbusDiControllerListener + Send>>,
    // port state cache
    port_state_vec: Vec<bool>,
}

impl ModbusListener for ModbusDiController {
    fn get_unit(&self) -> ModbusUnitSize {
        self.unit
    }

    fn get_port_num(&self) -> ModbusAddrSize {
        self.input_num
    }

    /// mount port object 
    fn mount_port(&mut self, address: ModbusAddrSize, port_to_mount: Box<dyn ModbusDiControllerListener + Send>) -> Result<(), DriverError> {
        self.mount_port_map.insert(address, port_to_mount);
        info!(LOG_TAG, "端口已挂载 address: {}", &address);
        Ok(())
    }

    /// read data from modbus and relay to port object
    /// - if data not change, do nothing
    /// - if data changed, notify port object
    /// - TODO 优化：可将缓存的数据和传入的数据保存为按位的整型，然后按位比较，可更快找到差异位置，然后通知下游
    fn notify_from_bus(&mut self, address: ModbusAddrSize, messages: Vec<bool>) -> Result<(), DriverError> {

        debug!(LOG_TAG, "从总线收到消息 address: {}, message: {:?}", &address, &messages);

        // 检查 address 是否有设备存在，如果没有设备，则忽略该条消息
        if !self.mount_port_map.contains_key(&address) {
            debug!(LOG_TAG, "未找到对应端口，忽略数据 address: {}", &address);
            return Ok(())
        }

        // check if data changed
        let port = self.mount_port_map.get(&address).ok_or(DriverError("DiController 接收到的消息，没有找到对应的端口".to_string()))?;
        for (i, message) in messages.iter().enumerate() {
            if self.port_state_vec[i] != *message {
                // if data changed, notify port object
                self.notify_port(address, *message)?;
                debug!(LOG_TAG, "端口状态发生变化，通知下游 address: {}, message: {}", &address, *message);
            }
        }
        self.port_state_vec = messages.clone();
        Ok(())
    }

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

impl ModbusDiController {
    pub fn new(device_id: &str, unit: ModbusUnitSize, input_num: ModbusAddrSize) -> Self {
        Self {
            device_id: device_id.to_string(),
            unit,
            input_num,
            mount_port_map: HashMap::new(),
            port_state_vec: vec![false; input_num as usize],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::modbus_di_port::ModbusDiPort;
    use crate::entity::bo::device_state_bo::{DeviceStateBo, DiStateBo, StateBoEnum};

    // 测试实例化并向上发送消息
    #[test]
    fn test_controller_notify_message() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut modbus_di_controller = ModbusDiController::new("test_controller", 1, 8);
        // 创建一个 port 设备
        let modbus_di_port = ModbusDiPort::new("test_di_port", 1, tx);

        modbus_di_controller.mount_port(1, Box::new(modbus_di_port)).unwrap();
        modbus_di_controller.notify_from_bus(1, vec![true, false, true, false, true, false, true, false]).unwrap();

        let state_bo = rx.recv().unwrap();

        match state_bo.state {
            StateBoEnum::Di(di_state) => {
                assert_eq!(di_state.on, true);
            }
            _ => {
                assert!(false);
            }
        }

    }
}