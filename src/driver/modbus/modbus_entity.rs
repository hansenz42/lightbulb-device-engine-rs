//! modbus 有关的内部使用实体

struct ModbusThreadCommandBo {
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

// 写单个线圈
struct WriteSingleBo {
    unit: u8,
    address: u16,
    value: u8,
}

// 写多个线圈一起写
struct WriteMultipleBo {
    unit: u8,
    start_address: u16,
    values: Vec<u8>
}