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
        let sender = self.get_upward_channel()?;
        sender.send(self.status())?;
        Ok(())
    }

    fn get_device_state_bo(&self) -> StateBo;

    /// 获取 manager
    fn get_upward_channel(&self) -> Result<Sender<DeviceStateBo>, Box<dyn Error>>;

    fn status(&self) -> DeviceStateBo;
}