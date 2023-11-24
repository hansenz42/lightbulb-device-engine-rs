//! modbus 总线设备类
//! modbus 上可以挂载多个输入输出单元，使用 unit 标识
//! 本类可以按照 unit 的顺序操作 modbus 设备。
//! 功能
//! - 支持读写 dmx 数据
//! - 多线程控制 dmx 写入

use std::{error::Error, rc::Rc, borrow::BorrowMut, cell::RefCell, sync::{Arc, Mutex, mpsc::Sender}};

use super::super::traits::bus::Bus;
use super::super::traits::device::Device;
use super::super::traits::master::Master;
use tokio_serial::SerialStream;
use tokio_modbus::{prelude::*, client::Context, Slave};
use std::collections::HashMap;
use crate::entity::bo::{device_config_bo::{ConfigBo}, device_state_bo::DeviceStateBo, device_state_bo::StateBoEnum};
use crate::common::error::DriverError;
use serde_json::Value;


/// Modbus 总线
pub struct ModbusBus {
    device_id: String,
    // 串口文件标识符
    serial_port: String,
    // 波特率
    baudrate: u32,
    // 已经注册的客户端哈希表
    slaves: HashMap<u8, Mutex<Context>>,
    // 上报通道
    upward_channel: Option<Sender<DeviceStateBo>>,
}

impl Master for ModbusBus {}

impl Device for ModbusBus {
    fn init(&self, device_config_bo: &ConfigBo) -> Result<(), DriverError> {
        // 检查串口是否可以打开
        let builder = tokio_serial::new(self.serial_port.as_str(), self.baudrate);
        let port = SerialStream::open(&builder).map_err(|e| {
            DriverError(format!("串口打开失败，serial_port {}, baud_rate{}, 异常: {}", self.serial_port.as_str(), self.baudrate, e))
        })?;
        Ok(())
    }

    fn get_category(&self) -> (String, String) {
        (String::from("bus"), String::from("modbus"))
    }

    fn get_device_id(&self) -> String {
        self.device_id.clone()
    }

    fn set_upward_channel(&mut self, sender: Sender<DeviceStateBo>) -> Result<(), DriverError> {
        self.upward_channel = Some(sender);
        Ok(())
    }

    fn get_upward_channel(&self) -> Option<Sender<DeviceStateBo>> {
        self.upward_channel.clone()
    }
}

impl Bus for ModbusBus {
    /// 检查当前的总线状态
    fn check(&self) -> Result<bool, Box<dyn Error>> {
        Ok(true)
    }

    /// 关闭当前的总线
    fn close(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// 重置总线
    fn reset(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl ModbusBus {
    pub fn new(device_id: String, serial_port: String, baudrate: u32) -> Self {
        Self {
            device_id,
            serial_port: serial_port,
            baudrate: baudrate,
            slaves: HashMap::new(),
            upward_channel: None
        }
}

    /// 注册一个 slave 设备
    pub fn register_slave(&mut self, unit: u8) -> Result<(), Box<dyn Error>> {
        let builder = tokio_serial::new(self.serial_port.as_str(), self.baudrate);
        let port = SerialStream::open(&builder)?;
        let slave = Slave(unit);
        let mut ctx = rtu::attach_slave(port, slave);
        // 将设备注册到哈希表
        self.slaves.insert(unit, Mutex::new(ctx));
        Ok(())
    }

    /// 解除一个 slave 设备
    pub async fn drop_slave(&mut self, unit: u8) -> Result<(), Box<dyn Error>> {
        let slave_option = self.slaves.remove(&unit);
        match slave_option {
            Some(slave) => {
                let mut ctx = slave.lock().map_err(|e| 
                    DriverError(format!("设备解除失败，unit: {}, 异常: {}", unit, e))
                )?;
                (* ctx).disconnect().await?;
                Ok(())
            },
            None => {
                Err("设备未注册".into())
            }
        }
    }

    /// 写单个线圈
    pub async fn write_coil(&self, unit: u8, address: u16, value: bool) -> Result<(), DriverError> {
        match self.slaves.get(&unit) {
            Some(ctx_ref) => {
                let mut ctx = (*ctx_ref).lock().map_err(|e| 
                    DriverError(format!("modbus 尝试写入失败，无法获取 context 加锁失败，unit: {}, 异常: {}", unit, e))
                )?;
                (* ctx).write_single_coil(address, value).await.map_err(|e| 
                    DriverError(format!("modbus 写入失败，unit: {}, 异常: {}", unit, e))
                )?;
                Ok(())
            },
            None => {
                Err(DriverError(format!("modbus 写入失败，unit: {}, 异常: {}", unit, "设备未注册")))
            }
        }
    }

    /// 写多个线圈
    pub async fn write_coils(& self, unit: u8, address: u16, values: Vec<bool>) -> Result<(), DriverError> {
        let ctx_option = self.slaves.get(&unit);
        match ctx_option {
            Some(ctx_ref) => {
                let mut ctx = (*ctx_ref).lock().map_err(
                    |e| DriverError(format!("modbus 尝试写入失败，无法获取 context 加锁失败，unit: {}, 异常: {}", unit, e))
                )?;
                ctx.write_multiple_coils(address, &values).await.map_err(
                    |e| DriverError(format!("modbus 写入失败，unit: {}, 异常: {}", unit, e))
                )?;
                Ok(())
            },
            None => {
                Err(DriverError(format!("modbus 写入失败，unit: {}, 异常: {}", unit, "设备未注册")))
            }
        }
    }

    /// 读单个寄存器
    pub async fn read_coil(&self, unit: u8, address: u16) -> Result<bool, DriverError> {
        match self.slaves.get(&unit) {
            Some(ctx_ref) => {
                let mut ctx = (*ctx_ref).lock().map_err(
                    |e| DriverError(format!("modbus 尝试读取失败，无法获取 context 加锁失败，unit: {}, 异常: {}", unit, e))
                )?;
                let ret = ctx.read_coils(address, 1).await.map_err(
                    |e| DriverError(format!("modbus 读取失败，unit: {}, 异常: {}", unit, e))
                )?[0];
                Ok(ret)
            },
            None => {
                Err(DriverError(format!("modbus 读取失败，unit: {}, 异常: {}", unit, "设备未注册")))
            }
        }
    }

    /// 读多个寄存器
    pub async fn read_coils(&self, unit: u8, address: u16, count: u16) -> Result<Vec<bool>, DriverError> {
        match self.slaves.get(&unit) {
            Some(ctx_ref) => {
                let mut ctx = (*ctx_ref).lock().map_err(
                    |e| DriverError(format!("modbus 尝试读取失败，无法获取 context 加锁失败，unit: {}, 异常: {}", unit, e))
                )?;
                let ret = ctx.read_coils(address, count).await.map_err(
                    |e| DriverError(format!("modbus 读取失败，unit: {}, 异常: {}", unit, e))
                )?;
                Ok(ret)
            },
            None => {
                Err(DriverError(format!("modbus 读取失败，unit: {}, 异常: {}", unit, "设备未注册")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        // let device = ModbusBus::new("test_device_id".to_string(),"/dev/ttyUSB0".to_string(), 9600);
    }
}