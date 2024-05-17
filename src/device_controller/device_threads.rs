use std::{
    any::Any,
    borrow::Borrow,
    collections::HashMap,
    fmt::format,
    process::exit,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::{
    common::error::DriverError,
    entity::dto::{
        device_command_dto::{DeviceCommandDto, CommandParamsEnum},
        device_report_dto::DeviceReportDto,
        device_state_dto::DeviceStateDto,
        server_state_dto::ServerStateDto,
    },
    mqtt_client::client::MqttClient,
};

use super::{
    device_factory::DeviceInstanceFactory,
    entity::{device_enum::DeviceRefEnum, device_po::DevicePo},
};
use crate::driver::modbus::traits::ModbusDoControllerCaller;
use crate::driver::traits::Commandable;
use crate::entity::dto::device_meta_info_dto::DeviceMetaInfoDto;
use crate::{debug, error, info, trace, warn};

const LOG_TAG: &'static str = "device_manager_threads";

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
            "device thread: device enum map length: {}",
            device_enum_map.len()
        );

        // 2. start running device
        match start_device(&device_enum_map) {
            Ok(_) => {}
            Err(e) => {
                error!(
                    LOG_TAG,
                    "device thread: cannot start device, error msg: {}", e
                );
                exit(1)
            }
        }
        info!(LOG_TAG, "device thread: successfully start all bus devices");

        loop {
            info!(LOG_TAG, "device thread: waitting for device command");
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
                master_modbus_ref.borrow_mut().start()?;
            }
            // run serial bus
            DeviceRefEnum::SerialBus(master_serial_ref) => {
                master_serial_ref.borrow_mut().start()?;
            }
            DeviceRefEnum::DmxBus(master_dmx_ref) => {
                master_dmx_ref.borrow_mut().start()?;
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
            do_port_ref_cell.borrow_mut().cmd(command_dto)?;
            Ok(())
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

/// device heartbeating thread
/// send heartbeating message with device info to flow server at a regular interval
pub fn heartbeating_thread(
    beat_interval_millis: u64,
    device_info_map: Arc<Mutex<HashMap<String, DeviceMetaInfoDto>>>,
    device_config_map: HashMap<String, DevicePo>,
    mqtt_client: Arc<Mutex<MqttClient>>,
) {
    let handle = thread::spawn(move || {
        info!(LOG_TAG, "heartbeating thread starting");
        loop {
            // 1. make device report message
            let mut report_dto_map: HashMap<String, DeviceReportDto> = HashMap::new();
            {
                let map_guard = device_info_map.lock().unwrap();
                for (device_id, device_info) in map_guard.iter() {
                    let report_dto = DeviceReportDto::from_device_meta_info(device_info);
                    report_dto_map.insert(device_id.clone(), report_dto);
                }
            }

            // 2. make server state, and send heartbeating message
            let server_state = ServerStateDto {
                device_config: device_config_map.clone(),
                device_status: report_dto_map,
            };
            info!(LOG_TAG, "heartbeating thread: send server state");
            {
                let mqtt_guard = mqtt_client.lock().unwrap();
                let ret = mqtt_guard.publish_heartbeat(server_state);
                match ret {
                    Ok(_) => {}
                    Err(e) => {
                        error!(LOG_TAG, "heartbeating thread: cannot publish server state to mqtt, will try in next beat, error msg: {}", e);
                    }
                }
            }

            // 3. sleep for beat_interval
            thread::sleep(std::time::Duration::from_millis(beat_interval_millis));
        }
    });
}

/// upward thread
/// used to report device state to upward controllers
/// the thread using tokio runtime because mqtt client is async
pub fn reporting_thread(
    state_report_rx: mpsc::Receiver<DeviceStateDto>,
    mqtt_client: Arc<Mutex<MqttClient>>,
) {
    thread::spawn(move || loop {
        info!(LOG_TAG, "waiting for device reporting message");
        let message = state_report_rx.recv();
        match message {
            Ok(device_state_bo) => {
                info!(
                    LOG_TAG,
                    "reporting thread: report message to mqtt: {:?}", &device_state_bo
                );
                {
                    let mqtt_guard = mqtt_client.lock().unwrap();
                    mqtt_guard
                        .publish_status(device_state_bo)
                        .expect("cannot publish upward message to mqtt");
                }
            }
            Err(e) => {
                warn!(
                    LOG_TAG,
                    "report thread exiting: device report channel error, msg: {}", e
                );
                return;
            }
        }
    });
}
