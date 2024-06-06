//! device enum
use std::{cell::RefCell, rc::Rc};

use crate::driver::{
    device::audio_output::AudioOutput,
    dmx::{dmx_bus::DmxBus, dmx_channel_device::DmxChannelDevice},
    modbus::{
        modbus_bus::ModbusBus, modbus_di_controller_coil::ModbusDiControllerCoil,
        modbus_di_port::ModbusDiPort, modbus_do_controller_coil::ModbusDoControllerCoil,
        modbus_do_port::ModbusDoPort,
    },
    serial::{serial_bus::SerialBus, serial_remote_controller::SerialRemoteController},
};

/// the base device type should managed by device manager
pub enum DeviceRefEnum {
    DmxBus(Rc<RefCell<DmxBus>>),
    DmxChannel(Rc<RefCell<DmxChannelDevice>>),
    SerialBus(Rc<RefCell<SerialBus>>),
    ModbusBus(Rc<RefCell<ModbusBus>>),
    ModbusDoController(Rc<RefCell<ModbusDoControllerCoil>>),
    ModbusDoPort(Rc<RefCell<ModbusDoPort>>),
    ModbusDiController(Rc<RefCell<ModbusDiControllerCoil>>),
    ModbusDiPort(Rc<RefCell<ModbusDiPort>>),
    SerialRemoteController(Rc<RefCell<SerialRemoteController>>),
    Audio(Rc<RefCell<AudioOutput>>),
}
