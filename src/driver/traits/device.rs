// 设备基类

pub trait Device {
    /// 设备初始化
    async fn init(&self) -> Result<(), Box<dyn Error>>;

    /// 设备启动
    async fn start(&self) -> Result<(), Box<dyn Error>>;

    /// 设备停止
    async fn stop(&self) -> Result<(), Box<dyn Error>>;

    /// 设备销毁
    async fn destroy(&self) -> Result<(), Box<dyn Error>>;

    /// 获取当前设备状态
    fn status(&self) -> Result<(), Box<dyn Error>>;
} 