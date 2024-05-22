use super::prelude::*;
use super::traits::ModbusDiControllerListener;
use crate::common::error::DriverError;
use crate::driver::traits::ReportUpward;
use crate::entity::dto::device_report_dto::DeviceReportDto;
use crate::entity::dto::device_state_dto::{StateToDeviceControllerDto, DiStateDto, StateDtoEnum};
use crate::{debug, error, info, trace, warn};
use std::env;
use std::sync::mpsc;

const DEVICE_TYPE: &str = "modbus_di_port";
const DEVICE_CLASS: &str = "operable";

const LOG_TAG: &str = "modbus_di_port";

/// modbus di port can be mounted to modbus controller
/// - has a upward channel to DeviceManager
/// - mount to controller object's port, when controller receieves data, the object will be reported
pub struct ModbusDiPort {
    device_id: String,
    address: ModbusAddrSize,
    upward_channel: mpsc::Sender<StateToDeviceControllerDto>,
    error_msg: Option<String>,
    error_timestamp: Option<u64>,
    last_update: Option<u64>,
}

impl ModbusDiControllerListener for ModbusDiPort {
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
            let state = StateDtoEnum::Di(DiStateDto { on: state_value });
            let device_state_dto = StateToDeviceControllerDto {
                device_id: self.device_id.clone(),
                device_class: DEVICE_CLASS.to_string(),
                device_type: DEVICE_TYPE.to_string(),
                status: DeviceReportDto {
                    state,
                    error_msg: self.error_msg.clone(),
                    error_timestamp: self.error_timestamp,
                    last_update: self.last_update,
                    active: true,
                }
            };
            let _ = self.notify_upward(device_state_dto)?;
            debug!(
                LOG_TAG,
                "di port state change, relay to upward, address: {}, message: {}", &self.address, state_value
            );
        }

        Ok(())
    }
}

impl ReportUpward for ModbusDiPort {
    fn get_upward_channel(&self) -> &mpsc::Sender<StateToDeviceControllerDto> {
        return &self.upward_channel;
    }

    /// CAUTION: di port do not call report(), it will report automatically during pooling
    fn report(&self) -> Result<(), DriverError> {
        Ok(())
    }
}

impl ModbusDiPort {
    pub fn new(
        device_id: &str,
        address: ModbusAddrSize,
        report_tx: mpsc::Sender<StateToDeviceControllerDto>,
    ) -> Self {
        ModbusDiPort {
            device_id: device_id.to_string(),
            address,
            upward_channel: report_tx,
            error_msg: None,
            error_timestamp: None,
            last_update: None,
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
        let state_bo: StateToDeviceControllerDto = rx.recv().unwrap();
    }
}
