//! modbus 有关的内部使用实体
use super::prelude::*;

// Modbus 线程指令对象，用于给线程下达指令用
#[derive(Debug)]
pub enum ModbusThreadCommandEnum {

    // send command to port
    WriteSingle(WriteSingleBo),
    WriteMulti(WriteMultiBo),

    // stop and close modbus threading
    Stop,
}

// 指令：写单个线圈
#[derive(Debug)]
pub struct WriteSingleBo {
    pub unit: ModbusUnitSize,
    pub address: ModbusAddrSize,
    pub value: bool,
}

// 指令：写多个线圈一起写
#[derive(Debug)]
pub struct WriteMultiBo {
    pub unit: ModbusUnitSize,
    pub start_address: ModbusAddrSize,
    pub values: Vec<bool>
}