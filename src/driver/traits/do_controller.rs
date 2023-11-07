/// 数字量输出控制器
pub trait DoController {

    /// 写单个寄存器地址
    fn write(&self, address: u8, value: u8) -> Result<(), Error>;

    /// 读单个寄存器地址
    fn read(&self, address: u8) -> Result<u8, Error>;

    /// 重置所有寄存器
    fn reset(&self) -> Result<(), Error>;
}