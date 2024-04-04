//! device enum
use std::{cell::RefCell, rc::Rc};

use crate::driver::{
    dmx::dmx_bus::DmxBus, modbus::{
        modbus_bus::ModbusBus, modbus_di_controller::ModbusDiController, modbus_di_port::ModbusDiPort, modbus_do_controller::ModbusDoController, modbus_do_port::ModbusDoPort
    }, serial::serial_bus::SerialBus
    
};

/// the base device type should managed by device manager
pub enum DeviceRefEnum {
    DmxBus(Rc<RefCell<DmxBus>>),
    SerialBus(Rc<RefCell<SerialBus>>),
    ModbusBus(Rc<RefCell<ModbusBus>>),
    ModbusDoController(Rc<RefCell<ModbusDoController>>),
    ModbusDoPort(Rc<RefCell<ModbusDoPort>>),
    ModbusDiController(Rc<RefCell<ModbusDiController>>),
    ModbusDiPort(Rc<RefCell<ModbusDiPort>>),
}

