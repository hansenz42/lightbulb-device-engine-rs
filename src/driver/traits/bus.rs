use std::error::Error;

/// 总线设备，拥有独立的执行线程
pub trait Bus {
    /// 检查当前的总线状态
    fn check(&self) -> Result<bool, Box<dyn Error>>;

    /// 关闭当前的总线
    fn close(&self) -> Result<(), Box<dyn Error>>;

    /// 重置总线
    fn reset(&self) -> Result<(), Box<dyn Error>>;
}