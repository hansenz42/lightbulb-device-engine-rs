use std::cell::RefCell;

use super::{modbus_bus::ModbusBus, prelude::*};
use crate::{common::error::DriverError, driver::traits::ReportUpward};

// ================= di ====================

/// the device that can listen to modbus event
/// it can be mounted to modbus controller
pub trait ModbusListener {
    fn get_unit(&self) -> ModbusUnitSize;

    fn get_port_num(&self) -> ModbusAddrSize;

    /// mount controller to modbus
    fn add_di_port(
        &mut self,
        address: ModbusAddrSize,
        di_port: Box<dyn ModbusDiControllerListener + Send>,
    ) -> Result<(), DriverError>;

    /// get data from modbus
    fn notify_from_bus(
        &mut self,
        address: ModbusAddrSize,
        values: Vec<bool>,
    ) -> Result<(), DriverError>;

    /// relay data to port device object
    fn notify_port(&self, address: ModbusAddrSize, values: bool) -> Result<(), DriverError>;
}

/// the device that can mount to modbus controller, and can report data to DeviceManager
pub trait ModbusDiControllerListener {
    fn get_address(&self) -> ModbusAddrSize;

    fn notify(&self, message: bool) -> Result<(), DriverError>;
}

// ================= do ====================

/// the device that can be mounted to modbus controller, and calling data to modbus
pub trait ModbusCaller: ReportUpward{
    fn get_unit(&self) -> ModbusUnitSize;

    fn get_output_num(&self) -> ModbusAddrSize;

    fn get_device_id(&self) -> String;

    fn get_port_state_vec_ref(&mut self) -> &mut Vec<bool>;

    fn set_port(&mut self, address: ModbusAddrSize, value: bool) -> Result<(), DriverError>;

    fn set_multi_ports(
        &mut self,
        address: ModbusAddrSize,
        values: &[bool],
    ) -> Result<(), DriverError>;

    fn write_one_port(&mut self, address: ModbusAddrSize, value: bool) -> Result<(), DriverError> {
        // check address range
        let device_id = self.get_device_id();
        if address >= self.get_output_num() {
            return Err(DriverError(format!("ModbusDoController: writing address out of range, device_id: {}, address: {}, value: {}", &device_id, address, value)));
        }

        // check if the value is different
        let port_state_vec = self.get_port_state_vec_ref();
        // update port state
        port_state_vec[address as usize] = value;
        let port_state = & port_state_vec[address as usize];

        if *port_state != value {
            let _ = self.set_port(address, value)?;
        }

        self.report()?;
        Ok(())
    }

    fn write_multi_ports(
        &mut self,
        address: ModbusAddrSize,
        values: &[bool],
    ) -> Result<(), DriverError> {
        // check address range
        let device_id = self.get_device_id();
        if address + values.len() as ModbusAddrSize > self.get_output_num() {
            return Err(DriverError(format!("ModbusDoController: writing address out of range, device_id: {}, address: {}, values: {:?}", &device_id, address, values)));
        }

        // check if the values are different
        let port_state_vec = self.get_port_state_vec_ref();
        let len = values.len();
        let port_state_slice = &port_state_vec[address as usize..(address as usize + len)];
        let mut is_diff = false;
        for i in 0..len {
            if port_state_slice[i as usize] != values[i as usize] {
                is_diff = true;
                break;
            }
        }

        // update port state
        for i in 0..len {
            port_state_vec[(address as usize + i) as usize] = values[i];
        }

        if is_diff {
            let _ = self.set_multi_ports(address, values)?;
        }

        self.report()?;
        Ok(())
    }
}

/// the device that can mount to modbus do controller
pub trait ModbusDoControllerCaller {
    fn get_address(&self) -> ModbusAddrSize;

    fn write(&self, value: bool) -> Result<(), DriverError>;
}
