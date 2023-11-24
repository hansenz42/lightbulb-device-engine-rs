//! 音频输出驱动
//! 使用：alsa 处理音频输出
//! 限制：只支持 i16 格式播放，如果是其他格式，请先转码。如果是使用上传方式，那么文件将会自动转吗
//! 功能：
//! - 根据文件路径播放音频
//! - 提供接口，音频可以暂停，停止和继续
//! - 提供混音，一个音频设备可以同时播放多个音频文件

use std::error::Error;
use std::sync::mpsc::Sender;
use std::thread;
use crate::common::error::DriverError;
use crate::entity::bo::device_state_bo::{DeviceStateBo, StateBoEnum, AudioStateBo, AudioFilePlayingBo};

use super::super::traits::playable::Playable;
use super::super::traits::device;
use rodio::{cpal, Device, Decoder, OutputStream, source::Source};
use rodio::cpal::traits::{HostTrait, DeviceTrait};

// 左右声道枚举
pub enum Channel {
    Left = 1,
    Right,
}

pub struct AudioOutput {
    device_id: String,
    // 声卡 id
    soundcard_id: String,
    // 使用的声道，每个 Output 支持单声道输出
    channel: Channel,
    // 上报通道
    upward_channel: Option<Sender<DeviceStateBo>>,
}

impl device::Device for AudioOutput {
    fn set_upward_channel(&mut self, sender: Sender<DeviceStateBo>) -> Result<(), DriverError> {
        self.upward_channel = Some(sender);
        Ok(())
    }

    fn get_upward_channel(&self) -> Option<Sender<DeviceStateBo>> {
        self.upward_channel.clone()
    }

    fn get_device_state_bo(&self) -> StateBoEnum {
        StateBoEnum::Audio(AudioStateBo{stream: vec![
            AudioFilePlayingBo{
                file_id: "test.mp3".to_string(),
                playing: true,
            }
        ]})
    }

    fn get_category(&self) -> (String, String) {
        (String::from("operable"), String::from("audio"))
    }

    fn get_device_id(&self) -> String {
        self.device_id.clone()
    }


}

impl Playable for AudioOutput {
    /// 根据文件路径播放
    fn play(&self, filename: String) -> Result<(), DriverError> {
        Ok(())
    }

    /// 如果正在播放，则暂停
    fn pause(&self) -> Result<(), DriverError> {
        Ok(())
    }

    fn stop(&self) -> Result<(), DriverError> {
        Ok(())
    }

    fn resume(&self) -> Result<(), DriverError> {
        Ok(())
    }
}

impl AudioOutput {
    pub fn new (device_id: String, soundcard_id: String, channel: Channel) -> AudioOutput {
        AudioOutput {
            device_id,
            soundcard_id,
            channel,
            upward_channel: None,
        }
    }

    /// 根据设备名称获取设备对象
    fn get_audio_device(device_name: String) -> Option<Device> {
        let host = cpal::default_host();
        let devices = host.output_devices().ok()?;

        for device in devices {
            if device.name().ok()?.starts_with(&device_name) {
                return Some(device)
            }
        }

        None
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_audio_output() {
    }
}