//! modbus 有关的内部使用实体
use super::prelude::*;

// Modbus 线程指令对象，用于给线程下达指令用
#[derive(Debug)]
pub enum ModbusThreadCommandEnum {

    // write coil command 
    WriteSingleCoil(WriteSingleCoilDto),
    WriteMultiCoils(WriteMultiCoilDto),

    // write register command
    WriteSingleRegister(WriteSingleRegisterDto),
    WriteMultiRegisters(WriteMultiRegistersDto),

    // stop and close modbus threading
    Stop,
}

#[derive(Debug)]
pub struct WriteSingleCoilDto {
    pub unit: ModbusUnitSize,
    pub address: ModbusAddrSize,
    pub value: bool,
}

#[derive(Debug)]
pub struct WriteMultiCoilDto {
    pub unit: ModbusUnitSize,
    pub start_address: ModbusAddrSize,
    pub values: Vec<bool>
}

#[derive(Debug)]
pub struct WriteSingleRegisterDto {
    pub unit: ModbusUnitSize,
    pub address: ModbusAddrSize,
    pub value: u16
}

#[derive(Debug)]
pub struct WriteMultiRegistersDto {
    pub unit: ModbusUnitSize,
    pub start_address: ModbusAddrSize,
    pub values: Vec<u16>
}