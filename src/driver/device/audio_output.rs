//! audio ouput driver
//! use: alsa libraray
//! limit: only support i16 format playback
//! 
//! function:
//! - play audio according to file path
//! - provide interface for audio pause, stop and resume
//! - mix audio files in one audio device

use crate::common::error::DriverError;
use crate::driver::traits::ReportUpward;
use crate::entity::dto::device_command_dto::{DeviceCommandDto, DeviceParamsEnum};
use crate::entity::dto::device_state_dto::{
    AudioFilePlayingDto, AudioStateDto, DeviceStateDto, StateDtoEnum,
};
use std::collections::HashMap;
use std::io::BufReader;
use std::sync::mpsc::Sender;

use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::source::{from_iter, ChannelVolume, FromIter, SamplesConverter, SineWave};
use rodio::{self, Sample, Sink};
use rodio::{cpal, source::Source, Decoder, OutputStream};
use std::fs::File;

const DEVICE_TYPE : &str = "audio";
const DEVICE_CLASS: &str = "operable";

// thie audio output use left or right channel
pub enum ChannelEnum {
    Left = 1,
    Right,
}

pub struct AudioOutput {
    device_id: String,
    soundcard_id: String,
    channel: ChannelEnum,
    report_tx: Sender<DeviceStateDto>,
    // active sink
    sink_map: HashMap<String, Sink>,
    // active output stream
    // CAUTION: need to save output stream to avoid output being recycled
    stream_map: HashMap<String, OutputStream>,
}

impl ReportUpward for AudioOutput {
    fn get_upward_channel(&self) -> &Sender<DeviceStateDto> {
        &self.report_tx
    }

    fn report(&self) -> Result<(), DriverError> {
        let mut playing_dto_list: Vec<AudioFilePlayingDto> = Vec::new();
        for filename in self.sink_map.keys() {
            let playing_dto = AudioFilePlayingDto {
                file_id: filename.clone(),
               playing: true,
            };
            playing_dto_list.push(playing_dto);
        }
        let state_dto = AudioStateDto {
            stream: playing_dto_list
        };
        self.report_tx
            .send(DeviceStateDto {
                device_class: DEVICE_CLASS.to_string(),
                device_type: DEVICE_TYPE.to_string(),
                device_id: self.device_id.clone(),
                state: StateDtoEnum::Audio(state_dto),
            })
            .map_err(|e| DriverError(format!("cannot report audio state, err: {}", e)))?;

        Ok(())
    }
}

impl AudioOutput {
    pub fn new(
        device_id: &str,
        soundcard_id: &str,
        channel: ChannelEnum,
        report_tx: Sender<DeviceStateDto>,
    ) -> AudioOutput {
        AudioOutput {
            device_id: device_id.to_string(),
            soundcard_id: soundcard_id.to_string(),
            channel,
            report_tx: report_tx,
            sink_map: HashMap::new(),
            stream_map: HashMap::new(),
        }
    }

    /// get audio device accoring to device name
    fn get_audio_device(&self, device_name: String) -> Option<rodio::Device> {
        let host = cpal::default_host();
        let devices = host.output_devices().ok()?;

        for device in devices {
            if device.name().ok()?.starts_with(&device_name) {
                return Some(device);
            }
        }
        None
    }

    pub fn cmd(&mut self, dto: DeviceCommandDto) -> Result<(), DriverError> {
        match dto.params {
            DeviceParamsEnum::Audio(audio_params) => {
                let action = dto.action;
                let filename = audio_params.filename;
                if action == "play" {
                    self.play(filename)?;
                } else if action == "pause" {
                    self.pause(filename)?;
                } else if action == "stop" {
                    self.stop(filename)?;
                } else if action == "resume" {
                    self.resume(filename)?;
                } else {
                    return Err(DriverError(format!("invalid action for audio device: {}", action)));
                }
            },
            _ => {
                return Err(DriverError(format!("invalid command data for audio device: {:?}", dto)));
            }
        }
        self.report()?;
        Ok(())
    }

    /// play audio according to file path
    /// playing files will be managed by sink
    /// CAUTION: there should be only one playing instance at the same time, if replaying the same file, previous instance should be stopped
    pub fn play(&mut self, filename: String) -> Result<(), DriverError> {
        // check if the file is already playing, if so, stop it first
        if self.sink_map.contains_key(&filename) {
            self.stop(filename.clone())?;
        }

        let file = BufReader::new(File::open(filename.clone()).map_err(|e| {
            DriverError(format!("cannot play file, unable to open file: {}, err: {}", &filename, e))
        })?);

        let decoder = Decoder::new(file).map_err(|e| {
            DriverError(format!("cannot decode file, unable to open file: {}, err: {}", &filename, e))
        })?;

        let source = ChannelVolume::new(decoder, vec![1.0f32, 0.0f32]);
        let device = self
            .get_audio_device(self.soundcard_id.clone())
            .ok_or_else(|| {
                DriverError(format!("cannot play file, unable to find device, soundcard_id: {}", &self.soundcard_id))
            })?;

        let (stream, stream_handle) = OutputStream::try_from_device(&device).map_err(|e| {
            DriverError(format!(
                "cannot get output handle, device name: {}, err: {}",
                &device
                    .name()
                    .unwrap_or(String::from("unable_to_get_device_name")),
                e
            ))
        })?;

        let sink: Sink = Sink::try_new(&stream_handle).map_err(|e| {
            DriverError(format!(
                "cannot get sink, device name: {}, err: {}",
                &device
                    .name()
                    .unwrap_or(String::from("unable_to_get_device_name")),
                e
            ))
        })?;

        sink.append(source);

        self.sink_map.insert(filename.clone(), sink);
        self.stream_map.insert(filename.clone(), stream);

        Ok(())
    }

    pub fn pause(&self, filename: String) -> Result<(), DriverError> {
        let sink = self
            .sink_map
            .get(&filename)
            .ok_or_else(|| DriverError(format!("cannot pause sink, unable to find filename: {}", &filename)))?;
        sink.pause();
        Ok(())
    }

    /// stop audio playing, and destroy current sink
    /// detail: the sink will be dropped
    pub fn stop(&mut self, filename: String) -> Result<(), DriverError> {
        let sink = self.sink_map.remove(&filename).ok_or_else(|| {
            DriverError(format!(
                "cannot remove sink, cannot find filename: {}",
                &filename
            ))
        })?;
        let stream = self.stream_map.remove(&filename).ok_or_else(|| {
            DriverError(format!(
                "cannot remove stream, cannot find filename: {}",
                &filename
            ))
        })?;
        sink.stop();
        Ok(())
    }

    pub fn resume(&self, filename: String) -> Result<(), DriverError> {
        let sink = self
            .sink_map
            .get(&filename)
            .ok_or_else(|| DriverError(format!("cannot resume sink, unable to find filename: {}", &filename)))?;
        sink.play();
        Ok(())
    }
}

mod tests {
    use std::sync::mpsc;
    use super::*;

    /// 测试音频播放文件
    #[test]
    fn test_audio_output() {
        let (tx, rx) = mpsc::channel();
        let mut audio_output = AudioOutput::new(
            "test",
            "plughw:CARD=PCH,DEV=0",
            ChannelEnum::Left,
            tx
        );
        let filename = String::from("/home/hansen/repo/lightbulb-device-engine-rs/file/188864511522626_file_example_WAV_2MG_1.wav");
        audio_output.play(filename.clone()).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(20));
        // audio_output.stop(String::from("/home/hansen/repo/lightbulb-device-engine-rs/file/188864511522626_file_example_WAV_2MG_1.wav")).unwrap();

        audio_output.stop(filename.clone()).unwrap();
        println!("stream stopped");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
