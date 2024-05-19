use std::{
    any::Any,
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    collections::HashMap,
    fmt::format,
    process::exit,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::{
    common::error::DriverError,
    entity::dto::{
        device_command_dto::DeviceCommandDto, device_meta_info_dto::DeviceStatusEnum,
        device_report_dto::DeviceReportDto, device_state_dto::DeviceStateDto,
        server_state_dto::ServerStateDto,
    },
    mqtt_client::client::MqttClient,
};

use super::super::{
    device_factory::DeviceInstanceFactory,
    entity::{device_enum::DeviceRefEnum, device_po::DevicePo},
};
use crate::driver::traits::Commandable;
use crate::entity::dto::device_meta_info_dto::DeviceMetaInfoDto;
use crate::{debug, error, info, trace, warn};

const LOG_TAG: &'static str = "device_thread";

/// device thread, use config to create device object, and send command to them
pub fn device_thread(
    state_report_tx_dummy: mpsc::Sender<DeviceStateDto>,
    command_rx: mpsc::Receiver<DeviceCommandDto>,
    device_po_list: Vec<DevicePo>,
    device_info_map: Arc<Mutex<HashMap<String, DeviceMetaInfoDto>>>,
) {
    let handle = thread::spawn(move || {
        // 1. make devices according to config
        let mut device_factory = DeviceInstanceFactory::new(state_report_tx_dummy);
        device_factory
            .make_devices(device_info_map, device_po_list)
            .unwrap();
        let device_enum_map = device_factory.get_device_map();
        info!(
            LOG_TAG,
            "device enum map length: {}",
            device_enum_map.len()
        );

        // 2. start running device
        match start_device(&device_enum_map) {
            Ok(_) => {}
            Err(e) => {
                error!(
                    LOG_TAG,
                    "cannot start device, error msg: {}", e
                );
                exit(1)
            }
        }
        info!(LOG_TAG, "successfully start all bus devices");

        loop {
            info!(LOG_TAG, "waitting for device command");
            let recv_message = command_rx.recv();
            match recv_message {
                Ok(command) => {
                    let device_id = &command.device_id;
                    info!(LOG_TAG, "command device, dto: {:?}", command);
                    // get device enum from map, if is none then print error msg
                    match device_enum_map.get(device_id) {
                        Some(device_enum) => {
                            info!(LOG_TAG, "sending command to device {:?}", command);
                            // send command to device
                            match command_device(device_enum, command) {
                                Ok(_) => {}
                                Err(e) => {
                                    error!(LOG_TAG, "command device error, error msg: {}", e);
                                    continue;
                                }
                            }
                        }
                        None => {
                            error!(
                                LOG_TAG,
                                "cannot send command to device, unable to find device_id: {} in device_enum_map", device_id
                            );
                            continue;
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        LOG_TAG,
                        "device worker thread exiting: device command downward channel closing, error msg: {}", e
                    );
                    return;
                }
            }
        }
    });
}


/// check all devices and run the threads if device has one
pub fn start_device(device_enum_map: &HashMap<String, DeviceRefEnum>) -> Result<(), DriverError> {
    for (device_id, device_ref) in device_enum_map {
        match device_ref {
            // run modbus
            DeviceRefEnum::ModbusBus(master_modbus_ref) => {
                let mut ref_cell = RefCell::borrow_mut(master_modbus_ref);
                ref_cell.start()?;
            }
            // run serial bus
            DeviceRefEnum::SerialBus(master_serial_ref) => {
                let mut ref_cell = RefCell::borrow_mut(master_serial_ref);
                ref_cell.start()?;
            }
            DeviceRefEnum::DmxBus(master_dmx_ref) => {
                let mut ref_cell = RefCell::borrow_mut(master_dmx_ref);
                ref_cell.start()?;
            }
            // run dmx bus
            _ => {}
        }
    }
    Ok(())
}

/// command device method
/// only limited type of device can be called.
/// selected type of device can only be called by specific params in device command dto
pub fn command_device(
    device_ref: &DeviceRefEnum,
    command_dto: DeviceCommandDto,
) -> Result<(), DriverError> {
    match device_ref {
        // do device
        DeviceRefEnum::ModbusDoPort(do_port_ref_cell) => {
            let mut ref_cell = RefCell::borrow_mut(do_port_ref_cell);
            ref_cell.cmd(command_dto)?;
            Ok(())
        },
        // audio device
        DeviceRefEnum::Audio(audio_ref_cell) => {
            let mut ref_cell = RefCell::borrow_mut(audio_ref_cell);
            ref_cell.cmd(command_dto)?;
            Ok(())
        }
        _ => {
            // do nothing
            Err(DriverError(format!(
                "not support device type, device type={:?}",
                device_ref.type_id()
            )))
        }
    }
}