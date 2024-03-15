use crate::common::error::DriverError;
use crate::entity::bo::serial_command_bo::SerialCommandBo;

/// 定义一个可侦听串口数据的特征

pub trait SerialListener {
    /// 串口 Bus 设备通知其绑定的设备
    fn notify(&self, data: SerialCommandBo) -> Result<(), DriverError> {Ok(())}
}