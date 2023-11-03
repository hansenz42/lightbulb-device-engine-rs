use std::error::Error;

use crate::device_controller::device_manager::DeviceManager;
use crate::entity::bo::device_state_bo::{StateBo, DeviceStateBo};
use std::sync::mpsc::Sender;

/// 定义一个挂载在总线上的设备
pub trait Logical {
    /// 设置上行信道
    fn set_upward_channel(&self, sender: Sender<DeviceStateBo>) -> Result<(), Box<dyn Error>>;

    /// 上报当前设备的状态
    fn notify(&self) -> Result<(), Box<dyn Error>> {
        let device_id = self.get_device_id();
        let device_class = self.get_device_class();
        let device_state_bo = DeviceStateBo {
            device_id,
            device_class,
            state: self.get_device_state_bo()
        };
        let sender = self.get_upward_channel()?;
        sender.send(device_state_bo)?;
        Ok(())
    }

    fn get_device_state_bo(&self) -> StateBo;

    /// 获取 manager
    fn get_upward_channel(&self) -> Result<Sender<DeviceStateBo>, Box<dyn Error>>;

    /// 获取 device_id
    fn get_device_id(&self) -> String;

    /// 获取 device_class
    fn get_device_class(&self) -> String;
}