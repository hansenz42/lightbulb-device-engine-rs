//! Modbus 内部线程方法集合
//! 启动线程，打开端口
//! 接收需要轮询的侦听接口信息，并将数据上报给 ModBusBus
//! 如有需要写入接口的数据，则在循环中断，并写入数据


use crate::common::error::DriverError;

use super::{entity::{ModbusThreadCommandEnum}, traits::ModbusListener};
use tokio_serial::SerialStream;
use tokio_modbus::{prelude::*, client::Context, Slave};
use std::{cell::RefCell, collections::HashMap, sync::mpsc::Receiver};
use super::prelude::*;
use crate::{info, warn, error, trace, debug};

const LOG_TAG: &str = "modbus_thread";
const MODBUS_POLLING_INTERVAL: u64 = 100;

/// 一个循环执行的端口守护线程
/// - 当有指令进入时，向下发送指令
/// - 当指令发送完毕或者没有指令时，则不断轮询接口提交给 Controller 对象，再由 Controller 对象向上游发送消息
pub async fn run_loop(
    serial_port: &str,
    baudrate: u32,
    command_rx: Receiver<ModbusThreadCommandEnum>,

    // di 控制器注册表，用于不间断轮询
    // 内部可变：因为需要调用 ModbusDigitalInputMountable 对象
    di_controller_map: HashMap<ModbusUnitSize, RefCell<Box<dyn ModbusListener + Send>>>

) -> Result<(), DriverError> {
    let mut context: Option<Context> = None;

    let env_mode = std::env::var("mode").unwrap_or("real".to_string());

    if env_mode == "dummy" {
        info!(LOG_TAG, "dummy mode, modbus port will not be open");
    } else {
        // 打开端口
        let builder = tokio_serial::new(serial_port, baudrate);
        let port = SerialStream::open(&builder).map_err(|e| {
            DriverError(format!("modbus worker, error, serial port cannot open, serial_port {}, baud_rate{}, exception: {}", serial_port, baudrate, e))
        })?;
        // 注册默认 Slave 以获得 Contenxt 对象
        let slave = Slave::broadcast();
        context = Some(rtu::attach_slave(port, slave));
    }

    loop {
        // 接收来自 rx 的消息，如果有消息，向下发送 modbus 指令
        if let Ok(command_enum) = command_rx.try_recv() {
            match command_enum {
                ModbusThreadCommandEnum::WriteSingle(write_single_bo) => {
                    
                    if env_mode == "dummy"  {
                        write_to_modbus_dummy(write_single_bo.unit, write_single_bo.address, write_single_bo.value)?;
                    } else {
                        let context_ref = context.as_mut();
                        write_to_modbus(&mut context_ref.unwrap(), write_single_bo.unit, write_single_bo.address, write_single_bo.value).await?;
                    }
                    
                },
                ModbusThreadCommandEnum::WriteMulti(write_multiple_bo) => {
                    if env_mode == "dummy"  {
                        write_multi_to_modbus_dummy(write_multiple_bo.unit, write_multiple_bo.start_address, write_multiple_bo.values.as_ref())?;
                    } else {
                        let context_ref = context.as_mut();
                        write_multi_to_modbus(&mut context_ref.unwrap(), write_multiple_bo.unit, write_multiple_bo.start_address, write_multiple_bo.values.as_ref()).await?;
                    }
                },
                ModbusThreadCommandEnum::Stop => {
                    info!(LOG_TAG, "modbus worker, stop command received, quitting");
                    return Ok(())
                }
            }
        } else {
            debug!(LOG_TAG, "modbus worker, no command received");
        }

        // 如果暂时没有消息，则对当前已经注册的设备轮询。
        // 对 controller_map 轮询
        for address in di_controller_map.keys() {
            let controller_cell = di_controller_map.get(address).ok_or(
                DriverError(format!("modbus worker，cannot get controller, exception: {}", address))
            )?;
            let mut controller = controller_cell.borrow_mut();
            let unit = controller.get_unit();
            let port_num = controller.get_port_num();
            
            let mut result = Ok(vec![]);

            if env_mode == "dummy" {
                result = read_from_modbus_dummy(unit, *address as ModbusAddrSize, port_num);
            } else {
                let context_ref = context.as_mut();
                // 读取端口状态，将状态传递给 controller
                result = read_from_modbus(&mut context_ref.unwrap(), unit,  *address as ModbusAddrSize, port_num).await;
            }
            
            match result {
                Ok(ret) => {
                    // 如果读取成功，就通知 controller
                    controller.notify_from_bus(*address as ModbusAddrSize, ret)?;
                },
                Err(e) => {
                    error!(LOG_TAG, "modbus worker 线程，读取 modbus 端口失败 {}", e)
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(MODBUS_POLLING_INTERVAL)).await;
    }
}


/// 从 modbus 端口一次性读取多个数据
pub async fn read_from_modbus(ctx: &mut Context, unit: ModbusUnitSize, address: ModbusAddrSize, count: ModbusAddrSize) -> Result<Vec<bool>, DriverError> {
    let slave = Slave(unit);
    ctx.set_slave(slave);
    let ret = ctx.read_coils(address, count).await.map_err(
        |e| DriverError(format!("modbus worker 线程，读取 modbus 端口失败，异常: {}", e))
    )?;
    Ok(ret)
}

pub fn read_from_modbus_dummy(unit: ModbusUnitSize, address: ModbusAddrSize, count: ModbusAddrSize) -> Result<Vec<bool>, DriverError>{
    info!(LOG_TAG, "模拟读取 modbus 端口，unit: {}, address: {}, count: {}", unit, address, count);
    Ok(vec![true; count as usize])
}

/// 向 modbus 端口写一个数据
pub async fn write_to_modbus(ctx: &mut Context, unit: ModbusUnitSize, address: ModbusAddrSize, value: bool) -> Result<(), DriverError> {
    let slave = Slave(unit);
    ctx.set_slave(slave);
    ctx.write_single_coil(address, value).await.map_err(
        |e| DriverError(format!("modbus worker 线程，写入 modbus 端口失败，异常: {}", e))
    )?;
    Ok(())
}

pub fn write_to_modbus_dummy(unit: ModbusUnitSize, address: ModbusAddrSize, value: bool) -> Result<(), DriverError> {
    info!(LOG_TAG, "模拟写入单个 modbus 端口，unit: {}, address: {}, value: {}", unit, address, value);
    Ok(())
}

/// 向 modbus 写多个数据
pub async fn write_multi_to_modbus(ctx: &mut Context, unit: ModbusUnitSize, start_address: ModbusAddrSize, values: &[bool]) -> Result<(), DriverError> {
    let slave = Slave(unit);
    ctx.set_slave(slave);
    ctx.write_multiple_coils(start_address, values).await.map_err(
        |e| DriverError(format!("modbus worker 线程，写入 modbus 端口失败，异常: {}", e))
    )?;
    Ok(())
}

pub fn write_multi_to_modbus_dummy(unit: ModbusUnitSize, start_address: ModbusAddrSize, values: &[bool]) -> Result<(), DriverError> {
    info!(LOG_TAG, "模拟写入多个 modbus 端口，unit: {}, start_address: {}, values: {:?}", unit, start_address, values);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::modbus_di_controller::ModbusDiController;
    use std::thread;
    use std::env;
    use crate::common::logger::init_logger;
    use crate::driver::modbus::entity::WriteSingleBo;

    // test reading, use di controller and port
    #[test]
    fn test_read() {
        env::set_var("mode", "dummy");
        let _ = init_logger();

        // 设置一个虚拟的 di 设备控制器
        let di_controller = ModbusDiController::new("test_controller", 1, 8);
        
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let serial_port = "/dev/ttyUSB0";
                let baudrate = 9600;
                let (tx, rx) = std::sync::mpsc::channel();
                let mut controller_map: HashMap<ModbusUnitSize, RefCell<Box<dyn ModbusListener + Send>>> = HashMap::new();
                controller_map.insert(1, RefCell::new(Box::new(di_controller)));
                let result = run_loop(serial_port, baudrate, rx, controller_map).await;
                assert!(result.is_ok());
            });
        });

        handle.join().unwrap();
    }

    // testing writing, use command object
    #[test]
    fn test_write() {
        env::set_var("mode", "dummy");
        let _ = init_logger();

        // 设置一个虚拟的 di 设备控制器
        let di_controller = ModbusDiController::new("test_controller", 1, 8);
        let (tx, rx) = std::sync::mpsc::channel();

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let serial_port = "/dev/ttyUSB0";
                let baudrate = 9600;
                
                let mut controller_map: HashMap<ModbusUnitSize, RefCell<Box<dyn ModbusListener + Send>>> = HashMap::new();
                controller_map.insert(1, RefCell::new(Box::new(di_controller)));
                let result = run_loop(serial_port, baudrate, rx, controller_map).await;
                
            });
        });

        tx.send(ModbusThreadCommandEnum::WriteSingle(WriteSingleBo{
            unit: 1,
            address: 1,
            value: true
        })).unwrap();

        handle.join().unwrap();
    }
}