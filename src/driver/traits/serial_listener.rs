use crate::common::error::DriverError;

/// 定义一个可侦听串口数据的特征

pub trait SerialListener {
    /// 串口 Bus 设备通知其绑定的设备
    fn notify(&self, data: Vec<u8>) -> Result<(), DriverError> {Ok(())}
}