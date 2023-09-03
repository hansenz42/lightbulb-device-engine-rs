
pub trait Bus {
    /// 检查当前的总线状态
    fn check(&self) -> Result<bool, Error>;

    /// 关闭当前的总线
    fn close(&self) -> Result<(), Error>;

    /// 重置总线
    fn reset(&self) -> Result<(), Error>;
}