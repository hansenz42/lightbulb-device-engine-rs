use super::entity::SerialThreadCommand;
use super::traits::SerialMountable;
use crate::common::error::DriverError;
use crate::{debug, error, info, trace, warn};
use std::cell::RefCell;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use tokio_util::{
    codec::{Decoder, Encoder, Framed},
	bytes::{self, Buf, BufMut}
};
use tokio_serial::SerialPortBuilderExt;
use tokio_serial::SerialStream;
use super::entity::SerialCommandBo;

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
    command_rx: Receiver<SerialThreadCommand>,
    // listeners
    listener_map: Vec<RefCell<Box<dyn SerialMountable + Send>>>,
) -> Result<(), DriverError> {
    
    let port = Arc::new(tokio_serial::new(serial_port, baudrate).open_native_async()
        .map_err(|e| {
            DriverError(format!("PANIC: thread thread, cannot open serial port: {}, err: {}", serial_port, e))
    })?);

    let (writer, reader) = LineCodec.framed(port).split();
    
    let rt = tokio::runtime::Runtime::new()
        .expect("PANIC: cannot start serial thread, cannot init tokio runtime");

    rt.block_on(async {
        let mut writer = writer;
        let mut reader = reader;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(LOOP_INTERVAL));
        loop {
            tokio::select! {
                // if there is data to write
                Some(command) = command_rx.recv() => {
                    match command {
                        SerialThreadCommand::Write(data) => {
                            trace!(LOG_TAG, "write data: {}", data);
                            if let Err(e) = writer.send(data).await {
                                error!(LOG_TAG, "write data error: {:?}", e);
                            }
                        }
                    }
                }
                // if there is no data to write, read data
                _ = interval.tick() => {
                    let mut buf = bytes::BytesMut::new();
                    match reader.try_next() {
                        Ok(Some(data)) => {
                            trace!(LOG_TAG, "read data: {:?}", data);
                            for listener in listener_map.iter() {
                                listener.borrow_mut().on_data_received(data);
                            }
                        }
                        Ok(None) => {
                            trace!(LOG_TAG, "read data: None");
                        }
                        Err(e) => {
                            error!(LOG_TAG, "read data error: {:?}", e);
                        }
                    }
                }
            }
        }
    });
    Ok(())
}

/// Line Codec for Serial
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct LineCodec;


impl Decoder for LineCodec {
	type Item = SerialCommandBo;
	type Error = std::io::Error;

	// decode received data, remove leading 0xfa and tailing 0xed, return data
	fn decode(&mut self, buf: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		// split at 0xed
		if let Some(n) = buf.iter().position(|b| *b == 0xed) {
			let line = buf.split_to(n);
			// remove leading 0xfa
			let (_, line) = line.split_at(1);
			buf.advance(1);
			// return data after parsing
			let command = line[0];
			let param_len = line[1] as usize;
			Ok(Some(SerialCommandBo {
				command: line[0],
				data: line[2..param_len + 2 + 1].to_vec()
			}))
		} else {
			Ok(None)
		}
	}
}

impl Encoder<SerialCommandBo> for LineCodec {
	type Error = std::io::Error;

	// input command and param，adding leading 0xfa and tailing 0xed
	fn encode(&mut self, item: SerialCommandBo, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
		dst.put_u8(0xfa);
		dst.put_u8(item.command);
		dst.put_u8(item.data.len() as u8);
		dst.put(item.data.as_ref());
		dst.put_u8(0xed);
		Ok(())
	}
}