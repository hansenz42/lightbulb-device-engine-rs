use super::entity::SerialDataBo;
use super::entity::SerialThreadCommand;
use super::traits::SerialMountable;
use crate::common::error::DriverError;
use crate::{debug, error, info, trace, warn};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt, TryStreamExt};
use std::cell::RefCell;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio_serial::SerialPortBuilderExt;
use tokio_serial::SerialStream;
use tokio_util::{
    bytes::{self, Buf, BufMut},
    codec::{Decoder, Encoder, Framed},
};

const LOG_TAG: &str = "serial_thread.rs | serial threading";

const LOOP_INTERVAL: u64 = 100;

/// start the serial thread
/// - write data if there is data to write
/// - if there is no data to write, read data
/// - set a read interval, loop reading
pub fn run_loop(
    serial_port: &str,
    baudrate: u32,
    // the sending command channel
    mut command_rx: Receiver<SerialThreadCommand>,
    // listeners
    listener_vec: Vec<RefCell<Box<dyn SerialMountable + Send>>>,
) -> Result<(), DriverError> {
    let port: Option<SerialStream> = None;
    let env_mode = std::env::var("mode").unwrap_or("real".to_string());
    let mut writer_opt: Option<SplitSink<Framed<SerialStream, _>, _>> = None;
    let mut reader_opt: Option<SplitStream<Framed<SerialStream, _>>> = None;

    if env_mode == "dummy" {
        info!(LOG_TAG, "dummy mode, serial port will not be open");
    } else {
        let port = tokio_serial::new(serial_port, baudrate)
            .open_native_async()
            .map_err(|e| {
                DriverError(format!(
                    "PANIC: thread thread, cannot open serial port: {}, err: {}",
                    serial_port, e
                ))
            })?;

        let (writer, reader) = LineCodec.framed(port).split();
        writer_opt = Some(writer);
        reader_opt = Some(reader);
    }

    let rt = tokio::runtime::Runtime::new()
        .expect("PANIC: cannot start serial thread, cannot init tokio runtime");

    let _ = rt.block_on(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(LOOP_INTERVAL));

        loop {
            tokio::select! {
                // if there is data to write
                command_opt = command_rx.recv() => {
                    if let Some(command) = command_opt {
                        match command {
                            SerialThreadCommand::Write(data) => {
                                trace!(LOG_TAG, "write data: {:?}", data);
                                if let Some(writer) = writer_opt.as_mut() {
                                    let _ = writer.send(data).await.map_err(|e| DriverError(format!("SerialBus send_data error: {}", e)))?;
                                }
                            },
                            SerialThreadCommand::Echo(data) => {
                                trace!(LOG_TAG, "echo data: {:?}", data);
                                for listener in listener_vec.iter() {
                                    let mut b = bytes::BytesMut::new();
                                    b.extend_from_slice(&data);
                                    let _ = listener.borrow_mut().notify(LineCodec.decode(&mut b).unwrap().unwrap());
                                }
                            },
                            SerialThreadCommand::Stop => {
                                info!(LOG_TAG, "stop signal received, exiting");
                                command_rx.close();
                                break;
                            }
                        }
                    } else {
                        warn!(LOG_TAG, "SerialBus command receiving channel received None data, tx could be closed");
                    }
                }

                // if there is no data to write, read data
                _ = interval.tick() => {
                    let mut buf = bytes::BytesMut::new();
                    if let Some(mut reader) = reader_opt.as_mut() {
                        match reader.try_next().await {
                            Ok(Some(data)) => {
                                trace!(LOG_TAG, "got data: {:?}", &data);
                                for listener in listener_vec.iter() {
                                    listener.borrow_mut().notify(data.clone());
                                }
                            }
                            Ok(None) => {
                                trace!(LOG_TAG, "read data: None");
                            }
                            Err(e) => {
                                error!(LOG_TAG, "read data error: {:?}", e);
                            }
                        }
                    } else {
                        warn!(LOG_TAG, "no reader available, skip reading");
                    }
                }
            }
        }
        Ok::<(), DriverError>(())
    });
    Ok(())
}

/// Line Codec for Serial Data
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct LineCodec;

impl Decoder for LineCodec {
    type Item = SerialDataBo;
    type Error = std::io::Error;

    // decode received data, remove leading 0xfa and tailing 0xed, return data
    fn decode(&mut self, buf: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // split at 0xe
        if let Some(n) = buf.iter().position(|b| *b == 0xed) {
            let line = buf.split_to(n);
            // remove leading 0xfa
            let (_, line) = line.split_at(1);
            buf.advance(1);
            // return data after parsing
            let command = line[0];
            let param_len = line[1] as usize;
            Ok(Some(SerialDataBo {
                command: line[0],
                data: line[2..param_len + 2].to_vec(),
            }))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<SerialDataBo> for LineCodec {
    type Error = std::io::Error;

    // input command and param，adding leading 0xfa and tailing 0xed
    fn encode(&mut self, item: SerialDataBo, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.put_u8(0xfa);
        dst.put_u8(item.command);
        dst.put_u8(item.data.len() as u8);
        dst.put(item.data.as_ref());
        dst.put_u8(0xed);
        Ok(())
    }
}
