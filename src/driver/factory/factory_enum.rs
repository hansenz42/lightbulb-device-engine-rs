//! 支持的设备枚举，避免使用动态特征
use super::modbus_factory::ModbusFactory;

enum DeviceFactoryEnum {
    // modbus 工厂
    ModbusFactory(ModbusFactory)
}