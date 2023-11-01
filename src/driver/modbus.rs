//! modbus 总线设备类
//! modbus 上可以挂载多个输入输出单元，使用 unit 标识
//! 本类可以按照 unit 的顺序操作 modbus 设备。
//! 功能
//! - 支持读写 dmx 数据
//! - 多线程控制 dmx 写入

use std::{error::Error, rc::Rc, borrow::BorrowMut, cell::RefCell};

use super::traits::bus::Bus;
use tokio_serial::SerialStream;
use tokio_modbus::{prelude::*, client::Context, Slave};
use std::collections::HashMap;


/// Modbus 总线
struct ModbusBus {
    // 串口文件标识符
    serial_port: String,
    // 波特率
    baudrate: u32,
    // 已经注册的客户端哈希表
    slaves: HashMap<u8, RefCell<Context>>,
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
    /// 创建一个 Modbus 总线设备
    fn new(serial_port: &str, baudrate: u32) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            serial_port: serial_port.to_string(),
            baudrate,
            slaves: HashMap::new(),
        })
    }

    /// 注册一个 slave 设备
    pub fn register_slave(&mut self, unit: u8) -> Result<(), Box<dyn Error>> {
        let builder = tokio_serial::new(self.serial_port.as_str(), self.baudrate);
        let port = SerialStream::open(&builder)?;
        let slave = Slave(unit);
        let mut ctx = rtu::attach_slave(port, slave);
        // 将设备注册到哈希表
        self.slaves.insert(unit, RefCell::new(ctx));
        Ok(())
    }

    /// 解除一个 slave 设备
    pub async fn drop_slave(&mut self, unit: u8) -> Result<(), Box<dyn Error>> {
        let slave_option = self.slaves.remove(&unit);
        match slave_option {
            Some(slave) => {
                slave.borrow_mut().disconnect().await?;
                Ok(())
            },
            None => {
                Err("设备未注册".into())
            }
        }
    }

    /// 写单个线圈
    pub async fn write_coil(&self, unit: u8, address: u16, value: bool) -> Result<(), Box<dyn Error>> {
        match self.slaves.get(&unit) {
            Some(ctx_ref) => {
                let mut ctx = (*ctx_ref).borrow_mut();
                (* ctx).write_single_coil(address, value).await?;
                Ok(())
            },
            None => {
                Err("设备未注册".into())
            }
        }
    }

    /// 写多个线圈
    pub async fn write_coils(& self, unit: u8, address: u16, values: Vec<bool>) -> Result<(), Box<dyn Error>> {
        let ctx_option = self.slaves.get(&unit);
        match ctx_option {
            Some(ctx_ref) => {
                let mut ctx = (*ctx_ref).borrow_mut();
                ctx.write_multiple_coils(address, &values).await?;
                Ok(())
            },
            None => {
                Err("设备未注册".into())
            }
        }
    }

    /// 读单个寄存器
    pub async fn read_coil(&self, unit: u8, address: u16) -> Result<bool, Box<dyn Error>> {
        match self.slaves.get(&unit) {
            Some(ctx_ref) => {
                let mut ctx = (*ctx_ref).borrow_mut();
                let ret = ctx.read_coils(address, 1).await?[0];
                Ok(ret)
            },
            None => {
                return Err("设备未注册".into());
            }
        }
    }

    /// 读多个寄存器
    pub async fn read_coils(&self, unit: u8, address: u16, count: u16) -> Result<Vec<bool>, Box<dyn Error>> {
        match self.slaves.get(&unit) {
            Some(ctx_ref) => {
                let mut ctx = (*ctx_ref).borrow_mut();
                let ret = ctx.read_coils(address, count).await?;
                Ok(ret)
            },
            None => {
                return Err("设备未注册".into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let device = ModbusBus::new("/dev/ttyUSB0", 9600);
    }
}