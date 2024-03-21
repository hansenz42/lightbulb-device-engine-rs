use super::entity::*;
use super::prelude::*;
use crate::common::error::DriverError;
use crate::{debug, info};
use dmx::DmxTransmitter;
use std::env;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread, time,
};

const LOG_TAG: &str = "dmx_thread.rs | dmx worker";
const LOOP_INTERVAL: u64 = 100;

pub fn run_loop(
    serial_port_str: &str,
    channel_data: [DmxValue; DmxChannelLen],
    // dmx 512 值接收器，当 dmx 值改变时接收变量
    downward_rx: mpsc::Receiver<DmxThreadCommandEnum>,
) -> Result<(), DriverError> {
    let dummy = env::var("dummy").unwrap_or("false".to_string());

    let mut dmx_port_option: Option<_> = None;

    if dummy == "true" {
        info!(LOG_TAG, "dmx worker thread started, dummy mode");    
    } else {
        dmx_port_option = Some(
            dmx::open_serial(serial_port_str)
                //.expect(format!("dmx worker 线程，无法打开端口： {serial_port_str}").as_str()),
                .map_err(|e| DriverError(format!("cannot open port, port:{}", serial_port_str)))?,
        );
        info!(
            LOG_TAG,
            "dmx worker thread started, transmitting, port: {}", serial_port_str
        );
    }

    let mut thread_channel_data = channel_data;

    loop {
        debug!(LOG_TAG, "sending dmx data...");
        match dmx_port_option {
            Some(ref mut dmx_port) => {
                dmx_port
                    .send_dmx_packet(thread_channel_data.as_ref())
                    .map_err(|e| DriverError(format!("cannot send data to port, port:{}", serial_port_str)))?;
            }
            None => {
                info!(LOG_TAG, "dmx worker thread, no port is available, skip this iter");
            }
        }
        thread::sleep(time::Duration::from_millis(10));
        
        match downward_rx.try_recv() {
            Ok(command) => {
                match command {
                    DmxThreadCommandEnum::SetChannel(set_channel_bo) => {
                        // check if there is new data, if there is, update dmx data
                        thread_channel_data.copy_from_slice(&set_channel_bo.channels);
                        info!(LOG_TAG, "get data from upward channel, relay to dmxbus: data = {:?}", &thread_channel_data);
                    }

                    DmxThreadCommandEnum::Stop => {
                        // check if the command is stop. if then, stop.
                        info!(LOG_TAG, "stop on STOP command");
                        break;
                    }
                }
            }
            Err(_) => {
                debug!(LOG_TAG, "no data, skip");
            }
        }
        
        thread::sleep(time::Duration::from_millis(LOOP_INTERVAL));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::logger::init_logger;
    use std::env;
    use std::thread;

    /// 测试线程执行
    #[test]
    fn test_thread() {
        env::set_var("dummy", "true");
        let _ = init_logger();

        let (downward_tx, downward_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            run_loop("COM1", [0; DmxChannelLen], downward_rx);
        });

        // downward_tx.send(DmxThreadCommandEnum::Stop).unwrap();
        let _ = downward_tx.send(DmxThreadCommandEnum::SetChannel(
            SetChannelBo {
                channels: [255; DmxChannelLen],
            }
        ));

        handle.join().unwrap();
    }
}
