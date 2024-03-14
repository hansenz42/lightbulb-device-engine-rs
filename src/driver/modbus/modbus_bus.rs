//! modbus 总线设备类
//! modbus 上可以挂载多个输入输出单元，使用 unit 标识
//! 本类可以按照 unit 的顺序操作 modbus 设备。
//! 功能：
//! - 维护一个线程：线程中跑一个 tokio 环境，用于设备调度
//! - 线程在空闲时会轮询所有的输入设备（如有），一旦发现数据变化，则通知后面的上行接口
//! - 写操作优先于读操作

use std::{borrow::BorrowMut, cell::RefCell, error::Error, rc::Rc, sync::{mpsc::Sender, Arc, Mutex}, thread};

use super::super::traits::interface::Interface;
use super::super::traits::device::Device;
use super::super::traits::master::Master;
use tokio_serial::SerialStream;
use tokio_modbus::{prelude::*, client::Context, Slave};
use std::collections::HashMap;
use crate::entity::bo::{device_command_bo::DeviceCommandBo, device_config_bo::ConfigBo, device_state_bo::{DeviceStateBo, StateBoEnum}};
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
    slave_pool: HashMap<u8, Mutex<Context>>,
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

impl ModbusBus {

    /// 打开端口，开始收发数据
    pub fn start_thread(&self) -> Result<(), DriverError> {
        // 创建设备执行下行通道
        let (tx_down, rx_down) = std::sync::mpsc::channel::<DeviceCommandBo>();

        // 创建消息上行通道
        let (tx_up, rx_up) = std::sync::mpsc::channel::<DeviceStateBo>();

        // 创建一个线程，运行 tokio 实例
        // 该线程根据通信通道中的数据，操作 Context 收发数据

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // 打开端口
                let builder = tokio_serial::new(self.serial_port.as_str(), self.baudrate);
                let port = SerialStream::open(&builder).expect("Modbus 新线程中接口打开失败");
                // let slave = Slave::broadcast();
                // let mut ctx = rtu::attach_slave(port, slave);

                
            });
        });
        
        Ok(())
    }

    pub fn new(device_id: &str, serial_port: &str, baudrate: u32) -> Self {
        Self {
            device_id: device_id.to_string(),
            serial_port: serial_port.to_string(),
            baudrate: baudrate,
            slave_pool: HashMap::new(),
            upward_channel: None,
        }
}

    /// 初始化 slave 模块并挂载到 SerialStream
    /// - 注意：一开始的挂载的 slave 默认为广播 slave
    pub fn init(&mut self) -> Result<(), DriverError> {
        let builder = tokio_serial::new(self.serial_port.as_str(), self.baudrate);
        let port = SerialStream::open(&builder).map_err(|e| {
            DriverError(format!("串口打开失败，serial_port {}, baudrate {}, 异常：{}", self.serial_port.as_str(), self.baudrate, e))
        })?;
        let slave = Slave::broadcast();
        let mut ctx = rtu::attach_slave(port, slave);
        self.context = Some(ctx);

        Ok(())
    }

    /// 检查当前 slave 并设置新的 slave_id
    pub fn set_slave(&mut self, slave_id: i16) -> Result<(), DriverError> {
        let mut ctx = self.context.as_mut().ok_or(DriverError("调用配置 slave 错误：modbus 上下文未初始化".to_string()))?;
        
        Ok(())
    }

    pub fn create_serial_stream(&mut self) -> Result<(), DriverError> {
        let builder = tokio_serial::new(self.serial_port.as_str(), self.baudrate);
        let port = SerialStream::open(&builder).map_err(|e| {
            DriverError(format!("串口打开失败，serial_port {}, baud_rate{}, 异常: {}", self.serial_port.as_str(), self.baudrate, e))
        })?;
        Ok(())
    }

    /// 解除一个 slave 设备
    pub async fn drop_slave(&mut self, unit: u8) -> Result<(), DriverError> {
        let slave_option = self.slave_pool.remove(&unit);
        match slave_option {
            Some(slave) => {
                let mut ctx = slave.lock().map_err(|e| 
                    DriverError(format!("设备解除失败，unit: {}, 异常: {}", unit, e))
                )?;
                (* ctx).disconnect().await.map_err(|e| 
                    DriverError(format!("设备解除失败，unit: {}, 异常: {}", unit, e))
                )?;
                Ok(())
            },
            None => {
                // Err("设备未注册".into())
                Err(DriverError(format!("设备解除失败，unit: {}, 异常: {}", unit, "设备未注册")))
            }
        }
    }

    /// 写单个线圈
    pub async fn write_coil(&self, unit: u8, address: u16, value: bool) -> Result<(), DriverError> {
        match self.slave_pool.get(&unit) {
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
        let ctx_option = self.slave_pool.get(&unit);
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
        match self.slave_pool.get(&unit) {
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
        match self.slave_pool.get(&unit) {
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
    use crate::common::logger::init_logger;

    #[test]
    fn test_new() {
        let _ = init_logger();

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let mut modbus_device = ModbusBus::new("test_device_id","/dev/ttyUSB1", 9600);
            modbus_device.write_coil(1, 1, true).await.unwrap();
            // modbus_device.write_coil(2, 1, True).unwrap();
            let ret = modbus_device.read_coil(1, 1).await.unwrap();
            assert_eq!(ret, true);
        });
    }
}