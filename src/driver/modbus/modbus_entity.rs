//! modbus 有关的内部使用实体
use super::prelude::*;

pub struct ModbusThreadCommandBo {
    command: ModbusThreadCommandEnum,
}

// Modbus 线程指令对象，用于给线程下达指令用
enum ModbusThreadCommandEnum {

    // 向端口下发数据
    WriteSingle(WriteSingleBo),
    WriteMultiple(WriteMultipleBo),

    // 停止线程并关闭端口
    Stop,
}

// 指令：写单个线圈
pub struct WriteSingleBo {
    unit: ModbusUnitSize,
    address: ModbusAddrSize,
    value: bool,
}

// 指令：写多个线圈一起写
pub struct WriteMultipleBo {
    unit: ModbusUnitSize,
    start_address: ModbusAddrSize,
    values: Vec<u8>
}