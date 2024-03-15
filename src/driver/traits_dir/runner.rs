use std::error::Error;

use crate::common::error::DriverError;

/// 需要独立运行的模块（拥有独立线程）
/// 例如：需要不断轮询的接口层设备
pub trait Runner {
    /// 检查当前的总线状态
    fn check(&self) -> Result<bool, DriverError>;

    /// 关闭当前的总线
    fn close(&self) -> Result<(), DriverError>;

    /// 重置总线
    fn reset(&self) -> Result<(), DriverError>;
}