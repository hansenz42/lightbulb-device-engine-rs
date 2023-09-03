/// 设备返回的状态
#[derive(Debug)]
pub struct DeviceStatus {
    // 设备 ID
    pub id: String,
    // 设备状态
    pub status: String,
}

/// 可以获取状态的设备
pub trait StatusDevice {
    /// 获取设备状态
    fn get_status(&self) -> Result<String, Error>;
}


/// 数字量输出控制器
pub trait DoController {

    /// 写单个寄存器地址
    fn write(&self, address: u8, value: u8) -> Result<(), Error>;

    /// 读单个寄存器地址
    fn read(&self, address: u8) -> Result<u8, Error>;

    /// 重置所有寄存器
    fn reset(&self) -> Result<(), Error>;
}

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