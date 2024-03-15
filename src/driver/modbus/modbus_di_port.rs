use super::traits::ModbusControllerMountable;
use std::sync::mpsc;
use super::prelude::*;
use crate::common::error::DriverError;
use crate::driver::traits::UpwardDevice;
use crate::entity::bo::device_state_bo::{DeviceStateBo, DiStateBo, StateBoEnum};

const DEVICE_TYPE: &str = "di";
const DEVICE_CLASS: &str = "operable";

/// modbus 上挂载的单个数字量输入接口
/// - 拥有一个上行通道，可以发送信息给 DeviceManager
pub struct ModbusDigitalInputPort {
    device_id: String,
    address: ModbusAddrSize,
    upward_channel: mpsc::Sender<DeviceStateBo>,
}

/// 对接口实现对控制器的可挂载特征
impl ModbusControllerMountable for ModbusDigitalInputPort {
    fn get_address(&self) -> ModbusAddrSize {
        self.address
    }

    // 制作返回给上游的数据结构体
    fn notify(&self, state_value: bool) -> Result<(), DriverError> {
        let state = StateBoEnum::Di(DiStateBo { on: state_value });
        let device_state_bo = DeviceStateBo {
            device_id: self.device_id.clone(),
            device_class: DEVICE_CLASS.to_string(),
            device_type: DEVICE_TYPE.to_string(),
            state: state,
        };
        let _ = self.notify_upward(device_state_bo)?;
        Ok(())
    }
}

impl UpwardDevice for ModbusDigitalInputPort {
    fn get_upward_channel(&self) -> &mpsc::Sender<DeviceStateBo> {
        return &self.upward_channel;
    }
}


impl ModbusDigitalInputPort {
    pub fn new(device_id: &str, address: ModbusAddrSize, upward_channel: mpsc::Sender<DeviceStateBo>) -> Self {
        ModbusDigitalInputPort {
            device_id: device_id.to_string(),
            address,
            upward_channel,
        }
    }
}

#[cfg(test)]
mod tests {
    // 注意这个惯用法：在 tests 模块中，从外部作用域导入所有名字。
    use super::*;

    /// 测试 di 设备的实例化，并测试发送消息
    #[test]
    fn test_di_device() {
        let (tx, rx) = mpsc::channel();
        let device_port = ModbusDigitalInputPort::new("di_1", 0, tx);
        device_port.notify(true).unwrap();
        let state_bo: DeviceStateBo = rx.recv().unwrap();
        match state_bo.state {
            StateBoEnum::Di(di_state) => {
                assert_eq!(di_state.on, true);
            }
            _ => {
                assert!(false);
            }
        }
    }
}