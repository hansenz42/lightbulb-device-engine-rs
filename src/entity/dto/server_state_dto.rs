use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::device_controller::entity::device_po::DevicePo;
use super::device_state_dto::DeviceStateDto;

pub struct ServerStateDto {
    device_config: HashMap<String, DevicePo>,
    device_status: HashMap<String, DeviceStateDto>
}
