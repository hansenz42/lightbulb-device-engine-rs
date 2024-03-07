//! dmx 总线设备类
//! dmx 总线可以控制多个带地址的设备
//! 
//! 简介
//! - 保存当前通道数据
//! - 创建独立的线程，开启端口并不断发送数据
//! - DmxBus 是一个控制器，负责和数据发送线程通信

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

use super::super::traits::interface::Interface;
use super::super::traits::master::Master;
use super::super::traits::device::Device;

const LOG_TAG : &str = "DeviceManager";

pub struct DmxBus {
    device_id: String,
    // 串口文件标识符
    serial_port: String,
    // 当前数据数组 512 u8 长度
    data: [u8; 512],
    // 线程运行标志
    thread_running_flag: Arc<Mutex<bool>>,
    // thread 发送通道句柄
    thread_tx: Option<mpsc::Sender<[u8; 512]>>,

    upward_channel: Option<Sender<DeviceStateBo>>,
}

impl Master for DmxBus {}

impl Interface for DmxBus {
    /// 检查当前的总线状态
    fn check(&self) -> Result<bool, Box<dyn Error>> {
        Ok(true)
    }

    /// 关闭当前的总线
    fn close(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// 重置总线
    fn reset(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl Device for DmxBus {
    fn set_upward_channel(&mut self, sender: Sender<DeviceStateBo>) -> Result<(), DriverError> {
        self.upward_channel = Some(sender);
        Ok(())
    }

    fn get_upward_channel(&self) -> Option<Sender<DeviceStateBo>> {
        self.upward_channel.clone()
    }

    fn get_device_state_bo(&self) -> StateBoEnum {
        StateBoEnum::DmxBus(DmxBusStateBo{
            debug_channels: self.data.to_vec(),
        })
    }

    fn get_category(&self) -> (String, String) {
        (String::from("bus"), String::from("dmxbus"))
    }

    fn get_device_id(&self) -> String {
        self.device_id.clone()
    }

    /// 新建线程并发送数据
    /// 新建线程以后，会将当前 data 发送给串口线程，后续修改数据时将通过通道将数据发现给串口线程
    fn start(&mut self) -> Result<(), DriverError> {
        let serial_port_str = self.serial_port.clone();

        // 准备线程使用的数据
        let thread_data = self.data.clone();
        let mut running_flag_ptr = self.thread_running_flag.lock().unwrap();
        *running_flag_ptr = true;
        let running_flag = Arc::clone(&self.thread_running_flag);

        // 创建通信通道
        let (tx, rx) = mpsc::channel();
        self.thread_tx = Some(tx);

        // 创建一个线程
        dmx_send_thread(thread_data, serial_port_str, running_flag, rx);

        info!(
            LOG_TAG,
            "dmx bus: start dmx bus, serial port: {}, data: {:?}",
            self.serial_port, self.data
        );

        Ok(())
    }
}

/// dmx 发送数据方法
/// 新建一个线程，返回线程句柄
fn dmx_send_thread(
    channel_data: [u8; 512], 
    serial_port_str: String,
    running_flag: Arc<Mutex<bool>>,
    rx: mpsc::Receiver<[u8; 512]>, 
) ->  Option<thread::JoinHandle<()>> {

    let handle = thread::spawn(move || {
        let mut dmx_port = dmx::open_serial(serial_port_str.as_str()).expect(format!("dmx worker 线程，无法打开端口： {serial_port_str}").as_str());
        info!(LOG_TAG, "dmx worker 线程启动: 已打开端口 {}, 开始传输", serial_port_str);
        let mut thread_channel_data = Box::new(channel_data);
        loop {
            debug!(LOG_TAG, "sending dmx data...");
            dmx_port.send_dmx_packet(thread_channel_data.as_ref()).expect(format!("dmx worker 线程: 无法发送数据 {serial_port_str}").as_str());
            thread::sleep(time::Duration::from_millis(10));

            // 检查是否停止，如果停止，则退出
            let running = running_flag.lock().expect("dmx worker 线程: 同步错误，运行标志位错误，检查代码！");
            if !*running {
                debug!(LOG_TAG, "dmx worker 线程停止");
                break;
            }

            // 接收是否有新的数据，如有，则更新当前的 data
            match rx.try_recv() {
                Ok(data) => {
                    thread_channel_data.copy_from_slice(&data);
                    info!(LOG_TAG, "dmx worker 线程，接收到数据 {:?}", &data);
                },
                Err(_) => {
                    debug!(LOG_TAG, "no data, skip");
                }  // 没有数据，继续发送
            }
        }
    });
    Some(handle)
    
}

impl DmxBus {

    /// 创建一个新的 dmx 总线设备
    pub fn new(device_id: &str, serial_port: &str) -> Self {
        Self {
            device_id: device_id.to_string(),
            serial_port: serial_port.to_string(),
            data: [0; 512],
            thread_running_flag: Arc::new(Mutex::new(false)),
            thread_tx: None,
            upward_channel: None,
        }
    }

    /// 设置当前单个地址上的数据
    fn set_channel(&mut self, address: u8, value: u8) -> Result<(), Box<dyn Error>> {
        self.data[address as usize] = value;
        match &self.thread_tx {
            Some(tx) => {
                tx.send(self.data).expect("dmx bus: send data to thread error");
                debug!(LOG_TAG, "dmx bus: send data to thread");
            },
            None => {
                error!(LOG_TAG, "dmx bus: thread tx is none");
            }
        }
        Ok(())
    }

    /// 设置多个地址上的数据
    fn set_channels(&mut self, address: u8, values: &[u8]) -> Result<(), Box<dyn Error>> {
        for i in 0..values.len() {
            self.data[address as usize + i] = values[i];
        }
        match &self.thread_tx {
            Some(tx) => {
                tx.send(self.data).expect("dmx bus: send data to thread error");
                debug!(LOG_TAG, "dmx bus: send data to thread");
            },
            None => {
                error!(LOG_TAG, "dmx bus: thread tx is none");
            }
        }
        Ok(())
    }

    /// 获取当前正在发送的数据
    fn get_data(&self, address: u8, length: u8) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut data = Vec::new();
        for i in address..address+length {
            data.push(self.data[i as usize]);
        }
        Ok(data)
    }

    /// 暂停向串口推送数据
    fn stop(&mut self) -> Result<(), Box<dyn Error + '_>> {
        let mut running_flag_ptr = self.thread_running_flag.lock()?;
        *running_flag_ptr = false;
        self.thread_tx = None;

        info!(
            LOG_TAG,
            "dmx bus: stop dmx bus, serial port: {}, data: {:?}",
            self.serial_port, self.data
        );

        Ok(())
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