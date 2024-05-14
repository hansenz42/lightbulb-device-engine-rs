use std::{
    any::Any,
    borrow::Borrow,
    collections::HashMap,
    fmt::format,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::{
    common::error::DriverError,
    driver::modbus::modbus_do_port::ModbusDoPort,
    entity::dto::{
        device_command_dto::{DeviceCommandDto, DeviceParamsEnum},
        device_state_dto::DeviceStateDto,
    },
    mqtt_client::client::MqttClient,
};

use super::{
    device_instance_factory::DeviceInstanceFactory,
    entity::{device_enum::DeviceRefEnum, device_po::DevicePo},
};
use crate::entity::dto::device_meta_info_dto::DeviceMetaInfoDto;
use crate::driver::modbus::traits::ModbusDoControllerCaller;
use crate::{debug, error, info, trace, warn};

const LOG_TAG: &'static str = "device_controller_thread";

/// device thread, use config to create device object, and send command to them
fn device_thread(
    upward_tx_dummy: mpsc::Sender<DeviceStateDto>,
    downward_rx: mpsc::Receiver<DeviceCommandDto>,
    device_po_list: Vec<DevicePo>,
    device_info_map: Arc<Mutex<HashMap<String, DeviceMetaInfoDto>>>,
) {
    // downward thread, send command to device
    let handle = thread::spawn(move || {
        // 1. make devices according to config
        let mut device_factory = DeviceInstanceFactory::new(upward_tx_dummy.clone());
        device_factory
            .make_devices(device_info_map, device_po_list)
            .unwrap();
        let device_enum_map = device_factory.get_device_map();

        loop {
            info!(LOG_TAG, "waitting for downward command");
            let recv_message = downward_rx.recv();
            match recv_message {
                Ok(commnad) => {
                    let device_id = &commnad.device_id;
                    // get device enum from map, if is none then print error msg
                    match device_enum_map.get(device_id) {
                        Some(device_enum) => {
                            // send command to device
                            match command_device(device_enum, commnad) {
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
                                "cannot find device_id: {} in device_enum_map", device_id
                            );
                            continue;
                        }
                    }
                }
                Err(e) => {
                    warn!(LOG_TAG, "downward worker channel closing, error msg: {}", e);
                    return;
                }
            }
        }
    });
}

/// device heartbeating thread
/// send heartbeating message with device info to flow server at a regular interval
fn heartbeating_thread(
    beat_interval_millis: u64,
    device_info_map: Arc<Mutex<HashMap<String, DeviceMetaInfoDto>>>,
    mqtt_client: Arc<Mutex<MqttClient>>
) {
    let handle = thread::spawn(move || {
        loop {
            // 1. send heartbeating message

            // 2. sleep for beat_interval
            thread::sleep(std::time::Duration::from_millis(beat_interval_millis));
        }
    });
}

/// upward thread
/// used to report device state to upward controllers
/// the thread using tokio runtime because mqtt client is async
fn upward_thread(upward_rx: mpsc::Receiver<DeviceStateDto>, mqtt_client: Arc<MqttClient>) {
    thread::spawn(move || {
        let rt =
            tokio::runtime::Runtime::new().expect("upward worker: cannot create tokio runtime");
        rt.block_on(async move {
            loop {
                info!(LOG_TAG, "waiting for upward message");
                let message = upward_rx.recv();
                match message {
                    Ok(device_state_bo) => {
                        info!(LOG_TAG, "upward message to mqtt: {:?}", &device_state_bo);
                        mqtt_client
                            .publish_status(device_state_bo)
                            .await
                            .expect("cannot publish upward message to mqtt");
                    }
                    Err(e) => {
                        warn!(LOG_TAG, "upward worker closed, channel error, msg: {}", e);
                        return;
                    }
                }
            }
        });
    });
}

/// command device method
/// only limited type of device can be called.
/// selected type of device can only be called by specific params in device command dto
fn command_device(
    device_ref: &DeviceRefEnum,
    command_dto: DeviceCommandDto,
) -> Result<(), DriverError> {
    match device_ref {
        // do device
        DeviceRefEnum::ModbusDoPort(do_port_ref_cell) => {
            if let DeviceParamsEnum::Do(do_param) = command_dto.params {
                let do_port = do_port_ref_cell.try_borrow().map_err(|e| {
                    DriverError(format!(
                        "device thread: borrow do_port error, error msg: {}",
                        e
                    ))
                })?;
                do_port.write(do_param.on.clone())?;
                Ok(())
            } else {
                Err(DriverError(format!(
                    "device thread: params is not do, params={:?}",
                    command_dto.params
                )))
            }
        }
        _ => {
            // do nothing
            Err(DriverError(format!(
                "device thread: not support device type, device type={:?}",
                device_ref.type_id()
            )))
        }
    }
}
