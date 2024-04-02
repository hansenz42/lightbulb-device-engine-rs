//! device enum
use std::rc::Rc;

use crate::driver::{
    dmx::dmx_bus::DmxBus,
    serial::serial_bus::SerialBus,
    modbus::{
        modbus_bus::ModbusBus,
        modbus_do_controller::ModbusDoController,
        modbus_do_port::ModbusDoPort
    },
    
};

/// the base device type should managed by device manager
pub enum DeviceRefEnum {
    DmxBus(Rc<DmxBus>),
    SerialBus(Rc<SerialBus>),
    ModbusBus(Rc<ModbusBus>),
    ModbusDoController(Rc<ModbusDoController>),
    ModbusDoPort(Rc<ModbusDoPort>),
}

