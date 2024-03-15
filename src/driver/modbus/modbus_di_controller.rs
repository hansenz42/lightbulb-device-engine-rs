use std::{collections::HashMap, hash::Hash};
use crate::common::error::DriverError;
use super::traits::{ModbusDigitalInputMountable, ModbusControllerMountable};
use super::prelude::*;

pub struct ModbusDiController {
    device_id: String,
    unit: ModbusUnitSize,
    // modbus 的端口数量
    input_num: ModbusAddrSize, 
    // modbus 控制器上搭载的接口信息
    mount_port_map:  HashMap<ModbusAddrSize, Box<dyn ModbusControllerMountable>>,
    // 当前状态缓存
    port_state_vec: Vec<bool>,
}

impl ModbusDigitalInputMountable for ModbusDiController {
    fn get_unit(&self) -> u8 {
        self.unit
    }

    fn get_port_num(&self) -> u16 {
        self.input_num
    }

    /// 挂载定义的端口
    fn mount_port(&mut self, address: ModbusAddrSize, port_to_mount: Box<dyn ModbusControllerMountable>) -> Result<(), DriverError> {
        // 将 ModbusControllerMounable 记录到 map 中
        self.mount_port_map.insert(address, port_to_mount);
        Ok(())
    }

    /// 接收从接口传来的数据
    /// - 比对，如果完全一样，则不动
    /// - 如果有地方不一样，则通知下游 port
    /// - TODO 优化：可将缓存的数据和传入的数据保存为整型，然后按位比较，可更快找到差异位置，然后通知下游
    fn notify_from_bus(&mut self, address: ModbusAddrSize, messages: Vec<bool>) -> Result<(), DriverError> {
        // 检查 address 是否有设备存在，如果没有设备，则忽略该条消息
        if !self.mount_port_map.contains_key(&address) {
            return Ok(())
        }

        // 比对数据
        let port = self.mount_port_map.get(&address).ok_or(DriverError("DiController 接收到的消息，没有找到对应的端口".to_string()))?;
        for (i, message) in messages.iter().enumerate() {
            if self.port_state_vec[i] != *message {
                // 发现不同，按照地址通知下游
                self.notify_port(address, *message)?;
            }
        }
        self.port_state_vec = messages.clone();
        Ok(())
    }

    fn notify_port(&self, address: ModbusAddrSize, message: bool) -> Result<(), DriverError> {
        // 检查 address 是否有设备存在，如果没有设备，则忽略该条消息
        if !self.mount_port_map.contains_key(&address) {
            return Ok(())
        }

        // 向设备发送消息
        let port: &Box<dyn ModbusControllerMountable> = self.mount_port_map.get(&address).ok_or(DriverError("DiController 向端口发送消息失败，没有找到对应的端口".to_string()))?;
        port.notify(message)?;
        Ok(())
    }
}

impl ModbusDiController {
    fn new(device_id: &str, unit: ModbusUnitSize, input_num: ModbusAddrSize) -> Self {
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
    use super::super::modbus_di_port::ModbusDigitalInputPort;
    use crate::entity::bo::device_state_bo::{DeviceStateBo, DiStateBo, StateBoEnum};

    // 测试实例化并向上发送消息
    #[test]
    fn test_controller_notify_message() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut modbus_di_controller = ModbusDiController::new("test_controller", 1, 8);
        // 创建一个 port 设备
        let modbus_di_port = ModbusDigitalInputPort::new("test_di_port", 1, tx);

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