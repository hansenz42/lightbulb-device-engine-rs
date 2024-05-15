use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::device_controller::entity::device_po::DevicePo;
use super::{device_report_dto::DeviceReportDto};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStateDto {
    pub device_config: HashMap<String, DevicePo>,
    pub device_status: HashMap<String, DeviceReportDto>
}
