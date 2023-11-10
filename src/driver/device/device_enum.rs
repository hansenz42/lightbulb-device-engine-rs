use super::modbus::ModbusBus;
use super::dmx_bus::DmxBus;
use super::serial_bus::SerialBus;

pub enum DeviceEnum {
    Modbus(ModbusBus),
    DmxBus(DmxBus),
    SerialBus(SerialBus),
}