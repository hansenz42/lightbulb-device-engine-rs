// 设备基类
use std::error::Error;
use std::sync::mpsc::Sender;
use async_trait::async_trait;
use serde_json::Value;
use crate::entity::bo::device_config_bo::{DeviceCreateBo, ConfigBo};
use crate::common::error::DriverError;
use crate::entity::bo::device_state_bo::{DeviceStateBo, StateBoEnum};


pub trait Device {
    /// 设备初始化
    /// - 检查当前设备是否配置正确
    /// - 初始化工作
    fn init(&self, device_config_bo: &ConfigBo) -> Result<(),DriverError> {Ok(())}

    /// 设备启动
    fn start(&self) -> Result<(), DriverError> {Ok(())}

    /// 设备停止
    fn stop(&self) -> Result<(), DriverError> {Ok(())}

    /// 设备销毁
    fn destroy(&self) -> Result<(), DriverError> {Ok(())}

    /// 默认实现：上报当前设备的状态
    fn notify(&self) -> Result<(), DriverError> {
        let sender = self.get_upward_channel().ok_or(DriverError("设备上行信道未初始化".to_string()))?;
        sender.send(self.status()).map_err(|err| DriverError("设备状态上报错误".to_string()))?;
        Ok(())
    }

    /// 默认实现：获取当前设备状态
    fn status(&self) -> DeviceStateBo {
        let bo = DeviceStateBo{
            device_id: self.get_device_id(),
            device_class: self.get_category().0,
            device_type: self.get_category().1,
            state: self.get_device_state_bo(),
        };
        bo
    }

    fn get_device_state_bo(&self) -> StateBoEnum {
        StateBoEnum::Empty
    }

    /// 给设备下达指令
    fn cmd(&self, action: String, param: Value) -> Result<(), DriverError> {Ok (())}

    /// 获取设备一级类目和二级类目（可初始化签名）
    fn get_category(&self) -> (String, String);

    /// 获取设备 device_id
    fn get_device_id(&self) -> String;

    /// 设置上行信道
    fn set_upward_channel(&mut self, sender: Sender<DeviceStateBo>) -> Result<(), DriverError>;

    /// 获取 manager
    fn get_upward_channel(&self) -> Option<Sender<DeviceStateBo>>;
} 