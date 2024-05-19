# 设备配置文档
## modbus 总线

```json
{
	"device_class": "bus",
	"device_type": "modbus_bus",
	"device_id": "modebus-total",
	"name": "modbus 总线",
	"room": "room",
	"description": "",
	"config": {
		"serial_port": "/dev/modbus0",
		"baudrate": 38400
	}
}
```

## 通用串口总线

```json
{
	"device_class": "bus",
	"device_type": "serial_bus",
	"device_id": "remote-total",
	"name": "通用串口总线",
	"room": "room",
	"description": "",
	"config": {
		"serial_port": "/dev/arduino-usb2",
		"baudrate": 9600
	}
}
```

## 数字输出/输入控制器

```json
{
	"device_class": "controller",
	"device_type": "modbus_do_controller",
	"device_id": "DO-24V-1",
	"name": "测试用数字输出控制器",
	"room": "room",
	"description": "",
	"config": {
		"unit": 1,
		"num": 32,
		"master_device_id": "some_modbus_device"
	}
}
```

输入控制器：type = modbus_di_controller

## 数字输出/输入端口

```json
{
	"device_class": "operable",
	"device_type": "modbus_do_port",
	"device_id": "do_output",
	"name": "某个do接口",
	"room": "room",
	"description": "",
	"config": {
		"address": 1,
		"master_device_id": "some_controller_device_id"
	}
}
```

输入端口：type = modbus_di_port

## 音频接口

```json
{
	"device_class": "operable",
	"device_type": "audio",
	"device_id": "playing-1",
	"name": "某个音频接口",
	"room": "room",
	"description": "",
	"config": {
		"soundcard_id": "usb-1",
		"channel": "left/right"
	}
}
```