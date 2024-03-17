//! dmx 总线设备类
//! dmx 总线可以控制多个带地址的设备
//! 
//! 功能
//! - 保存当前通道数据
//! - 创建独立的线程，开启端口并不断发送数据
//! - DmxBus 是一个控制器，负责和数据发送线程通信
//! - dmx 仅支持写而不支持读，所以只有下行数据而无上行数据

use dmx::{self, DmxTransmitter};
use serde_json::Value;
use crate::common;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use std::{thread, time, error::Error};
use std::time::Duration;
use crate::common::error::DriverError;
use crate::{info, warn, error, trace, debug};
use crate::entity::bo::device_state_bo::{StateBoEnum, DeviceStateBo, DmxBusStateBo};
use crate::common::logger::init_logger;
use super::prelude::{DmxValue, DmxChannelLen};
use super::dmx_thread::*;
use super::entity::*;

const LOG_TAG : &str = "DeviceManager";

pub struct DmxBus {
    device_id: String,
    // 串口文件标识符
    serial_port: String,
    // 当前数据数组 512 u8 长度
    data: [DmxValue; DmxChannelLen],
    // thread 发送通道句柄，只有在线程创建以后才可使用
    thread_tx: Option<mpsc::Sender<DmxThreadCommandEnum>>,

    upward_channel: Option<Sender<DeviceStateBo>>,
}

impl DmxBus {

    /// 创建一个新的 dmx 总线设备
    pub fn new(device_id: &str, serial_port: &str) -> Self {
        Self {
            device_id: device_id.to_string(),
            serial_port: serial_port.to_string(),
            data: [0; 512],
            thread_tx: None,
            upward_channel: None,
        }
    }

    /// 新建线程并发送数据
    /// 新建线程以后，会将当前 data 发送给串口线程，后续修改数据时将通过通道将数据发现给串口线程
    fn start(&mut self) -> Result<(), DriverError> {
        // 准备线程使用的数据
        let thread_data = self.data.clone();

        // 创建通信通道
        let (tx, rx) = mpsc::channel();
        self.thread_tx = Some(tx);

        let serial_port_str = self.serial_port.clone();

        // 创建一个线程
        let handle = thread::spawn(move || {
            run_loop(serial_port_str.as_str(), thread_data, rx);
        });

        info!(
            LOG_TAG,
            "dmx bus: start dmx bus, serial port: {}, data: {:?}",
            self.serial_port, self.data
        );

        Ok(())
    }

    /// 设置当前单个地址上的数据
    pub fn set_channel(&mut self, address: u8, value: u8) -> Result<(), DriverError> {
        self.data[address as usize] = value;
        self.send_channel_data_to_dmx()?;
        Ok(())
    }

    /// 设置多个地址上的数据
    pub fn set_channels(&mut self, address: u8, values: &[u8]) -> Result<(), DriverError> {
        for i in 0..values.len() {
            self.data[address as usize + i] = values[i];
        }
        self.send_channel_data_to_dmx()?;
        Ok(())
    }

    fn send_channel_data_to_dmx(&self) -> Result<(), DriverError> {
        match &self.thread_tx {
            Some(tx) => {
                tx.send(DmxThreadCommandEnum::SetChannel(
                    SetChannelBo {
                        channels: self.data.clone()
                    }
                )).expect("dmx bus: send data to thread error");
                debug!(LOG_TAG, "dmx bus: send data to thread");
                Ok(())
            },
            None => {
                Err(DriverError(format!("dmx bus: thread tx is none")))
            }
        }
    }

    /// 获取当前正在发送的数据
    fn get_data(&self, address: u8, length: u8) -> Result<Vec<DmxValue>, DriverError> {
        let mut data = Vec::new();
        for i in address..address+length {
            data.push(self.data[i as usize]);
        }
        Ok(data)
    }

    /// 向串口发送停止指令
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

    #[test]
    fn test_new() {
        let _ = common::logger::init_logger();
        let mut dmxbus = DmxBus::new("test_dmx_bus", "/dev/ttyUSB0");
        println!("dmx 总线启动");   
        dmxbus.start().unwrap();
        std::thread::sleep(Duration::from_secs(5));
        println!("修改 channel 的值");
        dmxbus.set_channel(2, 30);
        std::thread::sleep(Duration::from_secs(20));
    }
}