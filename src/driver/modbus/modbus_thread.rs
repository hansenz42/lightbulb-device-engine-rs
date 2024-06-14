//! Modbus 内部线程方法集合
//! 启动线程，打开端口
//! 接收需要轮询的侦听接口信息，并将数据上报给 ModBusBus
//! 如有需要写入接口的数据，则在循环中断，并写入数据

use crate::common::error::DriverError;

use super::prelude::*;
use super::{
    entity::ModbusThreadCommandEnum,
    traits::{ModbusControllerType, ModbusListener},
};
use crate::{debug, error, info, trace, warn};
use std::{cell::RefCell, collections::HashMap, sync::mpsc::Receiver};
use tokio_modbus::{client::Context, prelude::*, Slave};
use tokio_serial::SerialStream;

const LOG_TAG: &str = "modbus_thread";
const MODBUS_POLLING_INTERVAL: u64 = 100;

/// looping async function for commanding modbus port
/// - when a command is received, send the command
/// - when the controller is idle, it will poll all input devices (if any), and once the data changes, it will notify the upstream interface
pub async fn run_loop(
    serial_port: &str,
    baudrate: u32,
    command_rx: Receiver<ModbusThreadCommandEnum>,

    // di 控制器注册表，用于不间断轮询
    // 内部可变：因为需要调用 ModbusDigitalInputMountable 对象
    di_controller_map: HashMap<ModbusUnitSize, RefCell<Box<dyn ModbusListener + Send>>>,
) -> Result<(), DriverError> {
    let mut context: Option<Context> = None;

    let env_mode = std::env::var("mode").unwrap_or("real".to_string());

    if env_mode == "dummy" {
        info!(LOG_TAG, "dummy mode, modbus port will not be open");
    } else {
        // open the port
        let builder = tokio_serial::new(serial_port, baudrate);
        let port = SerialStream::open(&builder).map_err(|e| {
            DriverError(format!("modbus worker, error, serial port cannot open, serial_port {}, baud_rate{}, exception: {}", serial_port, baudrate, e))
        })?;
        // register slave with context
        let slave = Slave::broadcast();
        context = Some(rtu::attach_slave(port, slave));
    }

    loop {
        // send command to modbus thread
        if let Ok(command_enum) = command_rx.try_recv() {
            match command_enum {
                ModbusThreadCommandEnum::WriteSingleCoil(dto) => {
                    let context_ref = context.as_mut();
                    write_single_coil(&mut context_ref.unwrap(), dto.unit, dto.address, dto.value)
                        .await?;
                }
                ModbusThreadCommandEnum::WriteMultiCoils(dto) => {
                    let context_ref = context.as_mut();
                    write_multi_coils(
                        &mut context_ref.unwrap(),
                        dto.unit,
                        dto.start_address,
                        dto.values.as_ref(),
                    )
                    .await?;
                }
                ModbusThreadCommandEnum::WriteSingleRegister(dto) => {
                    let context_ref = context.as_mut();
                    write_single_register(
                        &mut context_ref.unwrap(),
                        dto.unit,
                        dto.address,
                        dto.value,
                    )
                    .await?;
                }
                ModbusThreadCommandEnum::WriteMultiRegisters(dto) => {
                    let context_ref = context.as_mut();
                    write_multi_registers(
                        &mut context_ref.unwrap(),
                        dto.unit,
                        dto.start_address,
                        dto.values.as_ref(),
                    )
                    .await?;
                }
                ModbusThreadCommandEnum::Stop => {
                    info!(LOG_TAG, "modbus worker, stop command received, quitting");
                    return Ok(());
                }
            }
        } else {
            debug!(LOG_TAG, "modbus worker, no command received");
        }

        // if there is no command received, it will poll all input devices
        // 对 controller_map 轮询
        for unit_key in di_controller_map.keys() {
            let controller_cell = di_controller_map.get(unit_key).ok_or(DriverError(format!(
                "modbus worker，cannot get controller, exception: {}",
                unit_key
            )))?;
            let mut controller = controller_cell.borrow_mut();
            let unit = controller.get_unit();
            let port_num = controller.get_port_num();
            let controller_type = controller.get_controller_type();

            let mut result = Ok(vec![]);
            let context_ref = context.as_mut();

            // read input value according to which type of controller
            match controller_type {
                ModbusControllerType::Coil => {
                    // read port status from modbus
                    result = read_coils(&mut context_ref.unwrap(), unit, 0, port_num).await;
                }
                ModbusControllerType::Register => {
                    let result_u16 =
                        read_input_registers(&mut context_ref.unwrap(), unit, 0, port_num).await;
                    result = result_u16.map(vec_u16_to_bool);
                }
            }

            match result {
                Ok(ret) => {
                    // relay data to controller
                    controller.notify_from_bus(*unit_key as ModbusAddrSize, ret)?;
                }
                Err(e) => {
                    error!(
                        LOG_TAG,
                        "modbus worker thread, reading modbus port failed {}", e
                    )
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(MODBUS_POLLING_INTERVAL)).await;
    }
}

// HELPER FUNCTIONS
pub fn vec_u16_to_bool(input: Vec<u16>) -> Vec<bool> {
    input.iter().map(|v| *v != 0).collect()
}

// MODBUS READING FUNCTIONS

pub async fn read_coils(
    ctx: &mut Context,
    unit: ModbusUnitSize,
    address: ModbusAddrSize,
    count: ModbusAddrSize,
) -> Result<Vec<bool>, DriverError> {
    let slave = Slave(unit);
    ctx.set_slave(slave);
    let ret = ctx.read_coils(address, count).await.map_err(|e| {
        DriverError(format!(
            "modbus worker thread, read modbus port failed, exc: {}",
            e
        ))
    })?;
    Ok(ret)
}

pub async fn read_holding_registers(
    ctx: &mut Context,
    unit: ModbusUnitSize,
    address: ModbusAddrSize,
    count: ModbusAddrSize,
) -> Result<Vec<u16>, DriverError> {
    let slave = Slave(unit);
    ctx.set_slave(slave);
    let ret = ctx
        .read_holding_registers(address, count)
        .await
        .map_err(|e| {
            DriverError(format!(
                "modbus worker thread, read modbus port failed, exc: {}",
                e
            ))
        })?;
    Ok(ret)
}

pub async fn read_input_registers(
    ctx: &mut Context,
    unit: ModbusUnitSize,
    address: ModbusAddrSize,
    count: ModbusAddrSize,
) -> Result<Vec<u16>, DriverError> {
    let slave = Slave(unit);
    ctx.set_slave(slave);
    let ret = ctx
        .read_input_registers(address, count)
        .await
        .map_err(|e| {
            DriverError(format!(
                "modbus worker thread, read modbus port failed, exc: {}",
                e
            ))
        })?;
    Ok(ret)
}

// MODBUS WRITING FUNCTIONS

pub async fn write_single_coil(
    ctx: &mut Context,
    unit: ModbusUnitSize,
    address: ModbusAddrSize,
    value: bool,
) -> Result<(), DriverError> {
    let slave = Slave(unit);
    ctx.set_slave(slave);
    ctx.write_single_coil(address, value).await.map_err(|e| {
        DriverError(format!(
            "modbus worker 线程，写入 modbus 端口失败，异常: {}",
            e
        ))
    })?;
    Ok(())
}

pub async fn write_multi_coils(
    ctx: &mut Context,
    unit: ModbusUnitSize,
    start_address: ModbusAddrSize,
    values: &[bool],
) -> Result<(), DriverError> {
    let slave = Slave(unit);
    ctx.set_slave(slave);
    ctx.write_multiple_coils(start_address, values)
        .await
        .map_err(|e| {
            DriverError(format!(
                "modbus worker 线程，写入 modbus 端口失败，异常: {}",
                e
            ))
        })?;
    Ok(())
}

pub async fn write_single_register(
    ctx: &mut Context,
    unit: ModbusUnitSize,
    address: ModbusAddrSize,
    value: u16,
) -> Result<(), DriverError> {
    let slave = Slave(unit);
    ctx.set_slave(slave);
    ctx.write_single_register(address, value)
        .await
        .map_err(|e| {
            DriverError(format!(
                "modbus worker 线程，写入 modbus 端口失败，异常: {}",
                e
            ))
        })?;
    Ok(())
}

pub async fn write_multi_registers(
    ctx: &mut Context,
    unit: ModbusUnitSize,
    start_address: ModbusAddrSize,
    values: &[u16],
) -> Result<(), DriverError> {
    let slave = Slave(unit);
    ctx.set_slave(slave);
    ctx.write_multiple_registers(start_address, values)
        .await
        .map_err(|e| {
            DriverError(format!(
                "modbus worker 线程，写入 modbus 端口失败，异常: {}",
                e
            ))
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::modbus_di_controller_coil::ModbusDiControllerCoil;
    use super::*;
    use crate::common::logger::init_logger;
    use crate::driver::modbus::entity::WriteSingleCoilDto;
    use std::env;
    use std::thread;

    // test reading, use di controller and port
    #[test]
    fn test_read() {
        env::set_var("mode", "dummy");
        let _ = init_logger();

        // 设置一个虚拟的 di 设备控制器
        // let di_controller = ModbusDiController::new("test_controller", 1, 8);

        // let handle = thread::spawn(move || {
        //     let rt = tokio::runtime::Runtime::new().unwrap();
        //     rt.block_on(async {
        //         let serial_port = "/dev/ttyUSB0";
        //         let baudrate = 9600;
        //         let (tx, rx) = std::sync::mpsc::channel();
        //         let mut controller_map: HashMap<ModbusUnitSize, RefCell<Box<dyn ModbusListener + Send>>> = HashMap::new();
        //         controller_map.insert(1, RefCell::new(Box::new(di_controller)));
        //         let result = run_loop(serial_port, baudrate, rx, controller_map).await;
        //         assert!(result.is_ok());
        //     });
        // });

        // handle.join().unwrap();
    }

    // testing writing, use command object
    #[test]
    fn test_write() {
        env::set_var("mode", "dummy");
        let _ = init_logger();

        // 设置一个虚拟的 di 设备控制器
        // let di_controller = ModbusDiController::new("test_controller", 1, 8);
        // let (tx, rx) = std::sync::mpsc::channel();

        // let handle = thread::spawn(move || {
        //     let rt = tokio::runtime::Runtime::new().unwrap();
        //     rt.block_on(async {
        //         let serial_port = "/dev/ttyUSB0";
        //         let baudrate = 9600;

        //         let mut controller_map: HashMap<ModbusUnitSize, RefCell<Box<dyn ModbusListener + Send>>> = HashMap::new();
        //         controller_map.insert(1, RefCell::new(Box::new(di_controller)));
        //         let result = run_loop(serial_port, baudrate, rx, controller_map).await;

        //     });
        // });

        // tx.send(ModbusThreadCommandEnum::WriteSingle(WriteSingleBo{
        //     unit: 1,
        //     address: 1,
        //     value: true
        // })).unwrap();

        // handle.join().unwrap();
    }
}
