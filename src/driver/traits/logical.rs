use std::error::Error;

use async_trait::async_trait;
use crate::device_controller::device_manager::DeviceManager;
use crate::entity::bo::state_bo::StateBo;

/// 定义一个挂载在总线上的设备
#[async_trait]
pub trait Logical {
    /// 注册 manager
    fn register_manager(&self, manager: DeviceManager) -> Result<(), Box<dyn Error>>;

    /// 将状态通知给 manager
    fn notify(&self, device_state_bo: &Box<dyn StateBo>) -> Result<(), Box<dyn Error>> {
        self.get_manager()?.notify(self.get_device_id(), device_state_bo);
        Ok(())
    }

    /// 获取 manager
    fn get_manager(&self) -> Result<DeviceManager, Box<dyn Error>>;

    /// 获取 device_id
    fn get_device_id(&self) -> &str;
}