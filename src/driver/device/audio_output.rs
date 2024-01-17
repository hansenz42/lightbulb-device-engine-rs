//! 音频输出驱动
//! 使用：alsa 处理音频输出
//! 限制：只支持 i16 格式播放，如果是其他格式，请先转码。如果是使用上传方式，那么文件应该要自动转码
//! 功能：
//! - 根据文件路径播放音频
//! - 提供接口，音频可以暂停，停止和继续
//! - 提供混音，一个音频设备可以同时播放多个音频文件

use std::collections::HashMap;
use std::error::Error;
use std::io::BufReader;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use crate::common::error::DriverError;
use crate::entity::bo::device_state_bo::{DeviceStateBo, StateBoEnum, AudioStateBo, AudioFilePlayingBo};

use super::super::traits::playable::Playable;
use super::super::traits::device::Device;
use std::fs::File;
use futures::sink;
use rodio::buffer::SamplesBuffer;
use rodio::{cpal, Decoder, OutputStream, source::Source};
use rodio::{self, Sink, Sample};
use rodio::cpal::traits::{HostTrait, DeviceTrait};
use rodio::source::{SamplesConverter, SineWave, FromIter, from_iter, ChannelVolume};
use serde_json::Value;


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
    // 活动中的 sink
    sink_map: HashMap<String, Sink>,
    // 活动中的 output stream：需要保存 output stream 避免被 rust 自动回收导致声音中断
    stream_map: HashMap<String, OutputStream>,
}

impl AudioOutput {
    pub fn new (device_id: String, soundcard_id: String, channel: Channel) -> AudioOutput {
        AudioOutput {
            device_id,
            soundcard_id,
            channel,
            upward_channel: None,
            sink_map: HashMap::new(),
            stream_map: HashMap::new(),
        }
    }

    /// 根据设备名称获取设备对象
   pub fn get_audio_device(&self, device_name: String) -> Option<rodio::Device> {
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

impl Device for AudioOutput {
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

    fn cmd(&mut self, action: String, param: Value) -> Result<(), DriverError> {
        if action == "play" {
            let filename = param["filename"].as_str().ok_or_else(|| {
                DriverError(format!("参数错误，缺少 filename"))
            })?;
            self.play(filename.to_string())?;
        } else if action == "pause" {
            let filename = param["filename"].as_str().ok_or_else(|| {
                DriverError(format!("参数错误，缺少 filename"))
            })?;
            self.pause(filename.to_string())?;
        } else if action == "stop" {
            let filename = param["filename"].as_str().ok_or_else(|| {
                DriverError(format!("参数错误，缺少 filename"))
            })?;
            self.stop(filename.to_string())?;
        } else if action == "resume" {
            let filename = param["filename"].as_str().ok_or_else(|| {
                DriverError(format!("参数错误，缺少 filename"))
            })?;
            self.resume(filename.to_string())?;
        } else {
            return Err(DriverError(format!("不支持的操作: {}", action)))
        }
        Ok(())
    }

}

impl Playable for AudioOutput {

    /// 根据文件路径播放（同一个声卡一个文件名只能有一个正在播放）
    /// 开始播放的文件，将加入到 sink 中做管理
    /// 注意：同一个时间只能有一个文件正在播放，如果遇到重复播放的文件，应该停止之前的播放实例
    fn play(&mut self, filename: String) -> Result<(), DriverError> {
        // 检查文件名是否正在播放，如果正在播放，则停止当前正在播放的内容
        if self.sink_map.contains_key(&filename) {
            self.stop(filename.clone())?;
        }

        let file = BufReader::new(File::open(filename.clone()).map_err(|e| {
            DriverError(format!("文件打开失败，文件名: {}, 异常: {}", &filename, e))
        })?);

        let decoder = Decoder::new(file).map_err(|e| {
            DriverError(format!("文件解码失败，文件名: {}, 异常: {}", &filename, e))
        })?;
        
        // let source = ChannelVolume::new(decoder, vec![1.0f32, 1.0f32]);

        // // 使用 ChannelVolume，不需要以下代码了
        // // 创建一个新的迭代器，将两个声道中的数据合并为到左声道，另一个声道写入零
        // let mut temp: i16 = 0;
        // let mut start_step: i16 = 0;
        // let mut start_target_step = 0;
        
        // // 声音数据的保存为交错模式
        // // 如果为左声道，则步长设定为 1 ，如果为右声道，则步长设定为 2
        // // 左声道输出模式为，首位加一个 0。举例：0 3 0 7 0 11 0 15
        // // 右声道输出模式为，首位加两个 0，举例：0 0 3 0 7 0 11 0 15
        // match self.channel {
        //     Channel::Left => {
        //         start_target_step = 1;
        //     },
        //     Channel::Right => {
        //         start_target_step = 2;
        //     }
        // }

        // let mono_source_iter = decoder.into_iter().map(|data| {
        //     if start_step < start_target_step {
        //         // 启动阶段，将数据加到 temp
        //         start_step += 1;
        //         temp += data;
        //         return 0;
        //     }
            
        //     if temp == 0 {
        //         // temp 无数据，则将数据保存到 temp
        //         temp = data;
        //         start_step += 1;
        //         return 0;  
        //     } else {
        //         // temp 有数据，则输出数据
        //         let ret = temp + data;
        //         temp = 0;
        //         start_step = 0;
        //         return ret;
        //     }
        // }); 

        let device = self.get_audio_device(self.soundcard_id.clone()).ok_or_else(|| {
            DriverError(format!("音频设备不存在，设备名: {}", &self.soundcard_id))
        })?;

        let (stream, stream_handle) = OutputStream::try_from_device(&device).map_err(|e| {
            DriverError(format!("获取音频流 Handle 失败，设备名: {}, 异常: {}", &device.name().unwrap_or(String::from("unable_to_get_device_name")), e))
        })?;

        let sink: Sink = Sink::try_new(&stream_handle).map_err(|e| {
            DriverError(format!("获取 sink 失败，设备名: {}, 异常: {}", &device.name().unwrap_or(String::from("unable_to_get_device_name")), e))
        })?;

        sink.append(decoder);

        // 记录到对象存储
        self.sink_map.insert(filename.clone(), sink);
        self.stream_map.insert(filename.clone(), stream);

        Ok(())
    }

    /// 根据文件路径暂停
    fn pause(&self, filename: String) -> Result<(), DriverError> {
        let sink = self.sink_map.get(&filename).ok_or_else(|| {
            DriverError(format!("文件不存在，文件名: {}", &filename))
        })?;
        sink.pause();
        Ok(())
    }

    /// 停止音频播放，并且 destroy 当前 sink
    /// 细节：该函数从 sink_map 中移除 sink，并且销毁 sink
    fn stop(&mut self, filename: String) -> Result<(), DriverError> {
        let sink = self.sink_map.remove(&filename).ok_or_else(|| {
            DriverError(format!("文件不存在，文件名: {}", &filename))
        })?;
        sink.stop();
        Ok(())
    }

    fn resume(&self, filename: String) -> Result<(), DriverError> {
        let sink = self.sink_map.get(&filename).ok_or_else(|| {
            DriverError(format!("文件不存在，文件名: {}", &filename))
        })?;
        sink.play();
        Ok(())
    }
}

mod tests {
    use super::*;

    /// 测试音频播放文件
    #[test]
    fn test_audio_output() {
        let mut audio_output = AudioOutput::new(String::from("test"), String::from("plughw:CARD=PCH,DEV=0"), Channel::Left);
        audio_output.play(String::from("/home/hansen/repo/lightbulb-device-engine-rs/file/188864511522626_file_example_WAV_2MG_1.wav")).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(20));
        // audio_output.stop(String::from("/home/hansen/repo/lightbulb-device-engine-rs/file/188864511522626_file_example_WAV_2MG_1.wav")).unwrap();
    }

    #[test]
    fn test_audio_output_directly() {
        // let host = cpal::default_host();
        // let devices = host.output_devices().unwrap();
        // let name = "plughw:CARD=PCH,DEV=0";
        // let mut selected_device: Option<rodio::Device> = None;

        // for device in devices {
        //     // use device
        //     if device.name().unwrap().starts_with(&name) {
        //         selected_device = Some(device)
        //     }
        // }
        // let selected_device = selected_device.unwrap();

        let mut audio_output = AudioOutput::new(String::from("test"), String::from("plughw:CARD=PCH,DEV=0"), Channel::Left);

        let selected_device = audio_output.get_audio_device(String::from("plughw:CARD=PCH,DEV=0")).unwrap();

        let (_stream, stream_handle) = OutputStream::try_from_device(&selected_device).unwrap();
        let file = BufReader::new(File::open("/home/hansen/repo/lightbulb-device-engine-rs/file/188864511522626_file_example_WAV_2MG_1.wav".to_string()).unwrap());
        let decoder = Decoder::new(file).unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        sink.append(decoder);
        std::thread::sleep(std::time::Duration::from_secs(20));
    }
}