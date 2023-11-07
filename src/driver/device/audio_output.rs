//! 音频输出驱动
//! 使用：alsa 处理音频输出
//! 限制：只支持 i16 格式播放，如果是其他格式，请先转码。如果是使用上传方式，那么文件将会自动转吗

use std::error::Error;
use std::thread;

// 左右声道枚举
enum Channel {
    Left = 1,
    Right,
}

pub struct AudioOutput {
    // 声卡 id
    soundcard_id: String,
    // 使用的声道，每个 Output 支持单声道输出
    channel: Channel    
}


mod tests {
    use super::*;

    #[test]
    fn test_audio_output() {
    }
}