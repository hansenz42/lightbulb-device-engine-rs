//! Modbus 内部线程方法集合
//! 启动线程，打开端口
//! 接收需要轮询的侦听接口信息，并将数据上报给 ModBusBus
//! 如有需要写入接口的数据，则在循环中断，并写入数据


use crate::common::error::DriverError;

use super::{modbus_entity::ModbusThreadCommandBo, traits::ModbusDigitalInputMountable};
use tokio_serial::SerialStream;
use tokio_modbus::{prelude::*, client::Context, Slave};
use std::{cell::RefCell, collections::HashMap, sync::mpsc::Receiver};
use crate::{info, warn, error, trace, debug};

const LOG_TAG: &str = "modbus_thread";

/// 发起一个循环执行的线程
/// 参数：port，向下调用的 device 接口，
pub async fn run_loop(
    serial_port: &str,
    baudrate: u32,
    command_rx: Receiver<ModbusThreadCommandBo>,
    // di 控制器注册表，用于不间断轮询
    // 内部可变：因为需要调用 ModbusDigitalInputMountable 对象
    di_controller_map: HashMap<u8, RefCell<Box<dyn ModbusDigitalInputMountable>>>
) -> Result<(), DriverError> {

    // 打开端口
    let builder = tokio_serial::new(serial_port, baudrate);
    let port = SerialStream::open(&builder).map_err(|e| {
        DriverError(format!("modbus worker 线程，串口打开失败，serial_port {}, baud_rate{}, 异常: {}", serial_port, baudrate, e))
    })?;
    // 注册默认 Slave 以获得 Contenxt 对象
    let slave = Slave::broadcast();
    let mut context = rtu::attach_slave(port, slave);

    loop {
        // 接收来自 rx 的消息，如果有消息，就指挥 modbus 控制器下发
        let command = command_rx.try_recv();

        // 如果暂时没有消息，则对当前已经注册的设备轮询。

        // 对 controller_map 轮询
        for key in di_controller_map.keys() {
            let controller_cell = di_controller_map.get(key).ok_or(
                DriverError(format!("modbus worker 线程，获取 controller 失败，异常: {}", key))
            )?;
            let mut controller = controller_cell.borrow_mut();
            let unit = controller.get_unit();
            let port_num = controller.get_port_num();
            
            // 读取端口状态
            let slave = Slave(unit);
            context.set_slave(slave);
            let result = read_from_modbus(&mut context, 0, port_num).await;
            match result {
                Ok(ret) => {
                    // 如果读取成功，就通知 controller
                    controller.notify_from_bus(0, ret)?;
                },
                Err(e) => {
                    error!(LOG_TAG, "modbus worker 线程，读取 modbus 端口失败 {}", e)
                }
            }
        }
    }
}


/// 从 modbus 端口读取数据
pub async fn read_from_modbus(ctx: &mut Context, address: u16, count: u16) -> Result<Vec<bool>, DriverError> {
    let ret = ctx.read_coils(address, count).await.map_err(
        |e| DriverError(format!("modbus worker 线程，读取 modbus 端口失败，异常: {}", e))
    )?;
    Ok(ret)
}