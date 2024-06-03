//! dmx modbus class
//! dmx 总线可以控制多个带地址的设备
//! 
//! 功能
//! - 保存当前通道数据
//! - 创建独立的线程，开启端口并不断发送数据
//! - DmxBus 是一个控制器，负责和数据发送线程通信
//! - dmx 仅支持写而不支持读，所以只有下行数据而无上行数据

use dmx::{self, DmxTransmitter};
use crate::driver::traits::ReportUpward;
use crate::entity::dto::device_report_dto::DeviceReportDto;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use std::{thread, time, error::Error};
use crate::common::error::DriverError;
use crate::{info, warn, error, trace, debug};
use crate::entity::dto::device_state_dto::{StateDtoEnum, StateToDeviceControllerDto, DmxBusStateDto};
use super::prelude::{DmxValue, DMX_CHANNEL_LEN};
use super::dmx_thread::*;
use super::entity::*;

const LOG_TAG : &str = "DmxBus";
const DEVICE_CLASS: &str = "bus";
const DEVICE_TYPE: &str = "dmx_bus";

pub struct DmxBus {
    device_id: String,
    serial_port: String,
    // data channel is 512 u8 length
    data: [DmxValue; DMX_CHANNEL_LEN],
    // thread command sending channel
    thread_tx: Option<mpsc::Sender<DmxThreadCommandEnum>>,
    report_tx: Sender<StateToDeviceControllerDto>,
    error_msg: Option<String>,
    error_timestamp: Option<u64>,
    last_update: Option<u64>,
}

impl ReportUpward for DmxBus {
    fn get_upward_channel(&self) -> &Sender<StateToDeviceControllerDto> {
        return &self.report_tx;
    }

    // report dmx channel state change to report channel
    fn report(&self) -> Result<(), DriverError> {
        let state = DmxBusStateDto {
            channel: Vec::from(self.data.clone())
        };
        self.notify_upward(StateToDeviceControllerDto {
            device_id: self.device_id.clone(),
            device_class: DEVICE_CLASS.to_string(),
            device_type: DEVICE_TYPE.to_string(),
            status: DeviceReportDto {
                error_msg: self.error_msg.clone(),
                error_timestamp: self.error_timestamp,
                last_update: self.last_update,
                active: true,
                state: StateDtoEnum::DmxBus(state)
            }
        })?;
        Ok(())
    }
}

impl DmxBus {

    /// create a new dmx bus device
    pub fn new(device_id: &str, serial_port: &str, report_tx: Sender<StateToDeviceControllerDto>) -> Self {
        Self {
            device_id: device_id.to_string(),
            serial_port: serial_port.to_string(),
            data: [0; 512],
            thread_tx: None,
            report_tx,
            error_msg: None,
            error_timestamp: None,
            last_update: None,
        }
    }

    /// start new data sending thread 
    /// after thread start, the data will be send to serial port through channel
    pub fn start(&mut self) -> Result<(), DriverError> {
        // prepare data for thread
        let thread_data = self.data.clone();

        // create channel
        let (tx, rx) = mpsc::channel();
        self.thread_tx = Some(tx);

        let serial_port_str = self.serial_port.clone();

        // create a thread loop
        let handle = thread::spawn(move || {
            let _ = run_loop(serial_port_str.as_str(), thread_data, rx);
        });

        info!(
            LOG_TAG,
            "dmx bus started, serial port: {}, data: {:?}",
            self.serial_port, self.data
        );

        Ok(())
    }

    /// set single channel on modbus bus
    pub fn set_channel(&mut self, address: u8, value: u8) -> Result<(), DriverError> {
        self.data[address as usize] = value;
        self.sync_channel_data_to_thread()?;
        self.report()?;
        Ok(())
    }

    /// set multiple channel on modbus bus
    pub fn set_channels(&mut self, address: u8, values: &[u8]) -> Result<(), DriverError> {
        for i in 0..values.len() {
            self.data[address as usize + i] = values[i];
        }
        self.sync_channel_data_to_thread()?;
        self.report()?;
        Ok(())
    }


    fn sync_channel_data_to_thread(&self) -> Result<(), DriverError> {
        match &self.thread_tx {
            Some(tx) => {
                tx.send(DmxThreadCommandEnum::SetChannel(
                    SetChannelBo {
                        channels: self.data.clone()
                    }
                )).map_err(|e| DriverError(format!("dmx bus: send data to thread error {:?}", e)))?;
                debug!(LOG_TAG, "dmx bus: send data to thread");
                Ok(())
            },
            None => {
                Err(DriverError(format!("dmx bus: thread tx is none")))
            }
        }
    }

    /// get the sending data of dmx bus
    fn get_data(&self, address: u8, length: u8) -> Result<Vec<DmxValue>, DriverError> {
        let mut data = Vec::new();
        for i in address..address+length {
            data.push(self.data[i as usize]);
        }
        Ok(data)
    }
    ///  stop the sending thread
    fn stop(&mut self) -> Result<(), DriverError> {
        match &self.thread_tx {
            Some(tx) => {
                tx.send(DmxThreadCommandEnum::Stop).expect("dmx bus: send stop command to thread error");
                info!(
                    LOG_TAG,
                    "dmx bus: stop dmx bus, serial port: {}, data: {:?}",
                    self.serial_port, self.data
                );
                Ok(())
            },
            None => {
                return Err(DriverError(format!("dmx bus: thread tx is none")).into());
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::common;
    use super::*;
    use std::env;

    #[test]
    fn test_new() {
        env::set_var("dummy", "true");

        let _ = common::logger::init_logger();
        // let mut dmxbus = DmxBus::new("test_dmx_bus", "/dev/ttyUSB0");
        // println!("dmx 总线启动");   
        // dmxbus.start().unwrap();
        // std::thread::sleep(Duration::from_secs(5));
        // println!("修改 channel 的值");
        // dmxbus.set_channel(2, 30);
        // std::thread::sleep(Duration::from_secs(20));
    }
}