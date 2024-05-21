# 设备新增步骤

- 写设备的 driver
- 写设备的 factory
- 注册 factory：device_factory.rs 中调用 factory
- 注册设备类型 Enum ： device_enum.rs
- 指令参数部分 Enum ： device_command_dto.rs
- 如需运行 run：device_thread.rs 中加入运行代码
- 如设备可发送指令 commandable ：
  - device_thread.rs 中加入命令分发代码
  - device_commander.rs 中加入 mqtt 过来的设备命令 parse 代码