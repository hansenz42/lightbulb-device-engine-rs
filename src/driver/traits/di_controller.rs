/// 数字量输入控制器
pub trait DiController {
    /// 读单个寄存器地址
    fn read(&self, address: u8) -> Result<u8, Error>;

    /// 在某个地址上注册一个侦听器
    /// 返回值：侦听器的 ID
    fn on(&self, address: u8, listener: Box<dyn Fn(u8) -> ()>) -> Result<String, Error>;

    /// 取消一个地址上的侦听器
    fn off(&self, address: u8, listener_id: &str) -> Result<(), Error>;
}