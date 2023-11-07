// 设备基类
use std::error::Error;
use async_trait::async_trait;
use serde_json::Value;
use crate::entity::bo::device_config_bo::DeviceConfigBo;
use crate::common::error::DriverError;
use crate::entity::bo::device_state_bo::{DeviceStateBo, StateBo};


#[async_trait]
pub trait Device {
    /// 设备初始化
    /// - 检查当前设备是否配置正确
    /// - 初始化工作
    fn init(&self, device_config_bo: &DeviceConfigBo) -> Result<(),DriverError>;

    /// 设备启动
    async fn start(&self) -> Result<(), DriverError> {Ok(())}

    /// 设备停止
    async fn stop(&self) -> Result<(), DriverError> {Ok(())}

    /// 设备销毁
    async fn destroy(&self) -> Result<(), DriverError> {Ok(())}

    /// 获取当前设备状态
    fn status(&self) -> DeviceStateBo {
        let bo = DeviceStateBo{
            device_id: self.get_device_id(),
            device_class: self.get_category().0,
            device_type: self.get_category().1,
            state: StateBo::Empty,
        };
        bo
    }

    /// 给设备下达指令
    async fn cmd(&self, action: &str, param: Value) -> Result<(), DriverError> {Ok (())}

    /// 获取设备一级类目和二级类目（可初始化签名）
    fn get_category(&self) -> (String, String);

    /// 获取设备 device_id
    fn get_device_id(&self) -> String;
} 