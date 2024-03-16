use std::collections::HashMap;

use super::prelude::*;
use super::traits::{ModbusDoMountable, ModbusDoControllerMountable};
use std::sync::mpsc;
use super::modbus_entity::{ModbusThreadCommandEnum, WriteMultipleBo, WriteSingleBo};
use crate::common::error::DriverError;


/// modbus 总线输出量控制器
/// - 记录当前输出量端口的状态
/// - 如果上游调用，则做一次对比，如果发现差异，则调用下游的 modbus 端口
pub struct ModbusDoController {
    device_id: String,
    unit: ModbusUnitSize,
    output_num: ModbusAddrSize,
    mount_port_map: HashMap<ModbusAddrSize, Box<dyn ModbusDoControllerMountable + Send>>,
    port_state_vec: Vec<bool>,
    modbus_bus_command_tx: mpsc::Sender<ModbusThreadCommandEnum>
}

impl ModbusDoMountable for ModbusDoController {
    fn get_unit(&self) -> ModbusUnitSize {
        self.unit
    }

    fn get_port_num(&self) -> ModbusAddrSize {
        self.output_num
    }

    fn write_one_port(&self, address: ModbusAddrSize, value:bool) -> Result<(), DriverError> {
        // 检查地址范围
        if address >= self.output_num {
            return Err(DriverError(format!("ModbusDoController: 写入的地址超出范围， device_id: {}, address: {}, value: {}", self.device_id, address, value)));
        }

        // 比对单个端口的值后下发
        let port_state = self.port_state_vec[address as usize];
        if port_state != value {
            self.modbus_bus_command_tx.send(ModbusThreadCommandEnum::WriteSingle(WriteSingleBo{
                unit: self.unit,
                address,
                value
            })).map_err( |e| {
                DriverError(format!("ModbusDoController: 无法写入到 modbus 接口，指令发送失败， device_id: {}, address: {}, value: {} error: {}", self.device_id, address, value, e))
            })?;
        }
        Ok(())
    }

    fn write_multi_port(&self, address: ModbusAddrSize, values: &[bool]) -> Result<(), DriverError> {
        // 检查地址范围
        if address + values.len() as ModbusAddrSize > self.output_num {
            return Err(DriverError(format!("ModbusDoController: 写入的地址超出范围， device_id: {}, address: {}, values: {:?}", self.device_id, address, values)));
        }
        
        // 比对多个端口的值
        let len = values.len();
        let port_state_slice = &self.port_state_vec[address as usize..(address as usize + len)];
        let mut is_diff = false;
        for i in 0..len {
            if port_state_slice[i as usize] != values[i as usize] {
                is_diff = true;
                break;
            }
        }

        if is_diff {
            self.modbus_bus_command_tx.send(ModbusThreadCommandEnum::WriteMultiple(WriteMultipleBo{
                unit: self.unit,
                start_address: address,
                values: values.to_vec()
            })).map_err( |e| {
                DriverError(format!("ModbusDoController: 无法写入到 modbus 接口，指令发送失败， device_id: {}, address: {}, values: {:?} error: {}", self.device_id, address, values, e))
            })?;
        }


        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // 测试写单接口
    #[test]
    fn test_write() {
        let (tx, rx) = mpsc::channel();

        let mut controller = ModbusDoController {
            device_id: "test".to_string(),
            unit: 1,
            output_num: 10,
            mount_port_map: HashMap::new(),
            port_state_vec: vec![false; 10],
            modbus_bus_command_tx: tx
        };

        let result = controller.write_one_port(0, true);

        let message = rx.recv().unwrap();
        println!("{:?}", message);
    }

    #[test]
    // 测试写多接口
    fn test_write_multi() {
        let (tx, rx) = mpsc::channel();

        let mut controller = ModbusDoController {
            device_id: "test".to_string(),
            unit: 1,
            output_num: 10,
            mount_port_map: HashMap::new(),
            port_state_vec: vec![false; 10],
            modbus_bus_command_tx: tx
        };

        let result = controller.write_multi_port(0, &[true, false, true]);

        let message = rx.recv().unwrap();
        println!("{:?}", message);
    }
}