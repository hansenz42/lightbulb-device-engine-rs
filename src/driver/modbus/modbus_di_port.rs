use super::prelude::*;
use super::traits::ModbusDiControllerMountable;
use crate::common::error::DriverError;
use crate::driver::traits::UpwardSendable;
use crate::entity::bo::device_state_bo::{DeviceStateBo, DiStateBo, StateBoEnum};
use crate::{debug, error, info, trace, warn};
use std::env;
use std::sync::mpsc;

const DEVICE_TYPE: &str = "di";
const DEVICE_CLASS: &str = "operable";

const LOG_TAG: &str = "modbus_di_port";

/// modbus di port can be mounted to modbus controller
/// - has a upward channel to DeviceManager
/// - mount to controller object's port, when controller receieves data, the object will be reported
pub struct ModbusDiPort {
    device_id: String,
    address: ModbusAddrSize,
    upward_channel: mpsc::Sender<DeviceStateBo>,
}

impl ModbusDiControllerMountable for ModbusDiPort {
    fn get_address(&self) -> ModbusAddrSize {
        self.address
    }

    fn notify(&self, state_value: bool) -> Result<(), DriverError> {
        let env_mode = env::var("mode").unwrap_or("real".to_string());

        if env_mode == "dummy" {
            info!(
                LOG_TAG,
                "modbus di port is in dummy mode, receive remote data: {:?}", &state_value
            );
        } else {
            let state = StateBoEnum::Di(DiStateBo { on: state_value });
            let device_state_bo = DeviceStateBo {
                device_id: self.device_id.clone(),
                device_class: DEVICE_CLASS.to_string(),
                device_type: DEVICE_TYPE.to_string(),
                state: state,
            };
            let _ = self.notify_upward(device_state_bo)?;
            debug!(
                LOG_TAG,
                "di port state change, relay to upward, address: {}, message: {}", &self.address, state_value
            );
        }

        Ok(())
    }
}

impl UpwardSendable for ModbusDiPort {
    fn get_upward_channel(&self) -> &mpsc::Sender<DeviceStateBo> {
        return &self.upward_channel;
    }
}

impl ModbusDiPort {
    pub fn new(
        device_id: &str,
        address: ModbusAddrSize,
        upward_channel: mpsc::Sender<DeviceStateBo>,
    ) -> Self {
        ModbusDiPort {
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

    /// test di device and sending data
    #[test]
    fn test_di_device() {
        let (tx, rx) = mpsc::channel();
        let device_port = ModbusDiPort::new("di_1", 0, tx);
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
