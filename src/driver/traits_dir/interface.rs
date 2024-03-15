use std::error::Error;

use crate::common::error::DriverError;

/// 接口层设备
/// - 数据收发
/// - 状态维护
/// - 一般会有自己的独立管理线程，控制收发
pub trait Interface {
    /// 检查当前的总线状态
    fn check(&self) -> Result<bool, DriverError>;

    /// 关闭当前的总线
    fn close(&self) -> Result<(), DriverError>;

    /// 重置总线
    fn reset(&self) -> Result<(), DriverError>;
}