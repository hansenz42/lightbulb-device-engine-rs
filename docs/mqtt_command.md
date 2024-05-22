# MQTT 协议说明

## topic 说明

```
{command}/{application_name}/{scenario_name}/{srever_type}/{server_id}/{device_type}/{device_id}
```
- command：指令
  - cmd：对设备下指令
  - status：设备上报状态
  - broadcast：上位机对所有下位设备广播消息
- application_name：应用名称，config yaml 配置文件中定义
- scenario_name：场景名称，config yaml 配置文件中定义
- server_type：设备服务器类型，一般为 deviceserver
- server_id：设备服务器 id
- device_type：设备类型，不同类型的设备有不同的参数类型
- device_id：设备 id

## 协议结构体

```json
{
	"code":200,
	"msg":"some_message",
	"source_type":"upstream_server",
	"source_id":"upstream_server_id",
	"target_type":"device_server",
	"target_id":"device_id",
	"session_id":"random_session_id",
	"timestamp":1714044693341,
	"data":{"action":"on","param":null}
}
```
- code：状态码，200 表示正常，参考 http 状态码
- msg：文字消息，可以是错误信息等
- source_type：发送方的类型 upstream_server 表示上游服务器
- source_id：发送方 id
- target_type：接收方类型 device_server 表示设备服务器（本服务）
- target_id：接收方 id
- session_id：随机字符串，标识该消息的 id
- data：结构体数据

## 发送：设备状态变化
Topic
```
status/{application_name}/{scenario_name}/deviceserver/{server_id}/{device_type}/{device_id}
```

Payload
```json
{
    ...
    "data": {
        "active": true,
        "error_msg": null,
        "error_timestamp": 1714044693341,
        "last_update": 1714044693341,
        "state": {
            ...
        }
    }
}
```
- state：设备不同结构体不同

## 发送：服务状态变化
Topic
```
status/{application_name}/{scenario_name}/deviceserver/{server_id}
```

Payload
```json
{
    ...
    "data": {
        "device_config": {
            "device_1": {
                "device_id": "xxx",
                "device_class": "xxx",
                "device_type": "xxxx",
                "name": "xxx",
                "description": "xxxx",
                "config": {
                    ...
                }
            }
        },
        "device_status": {
            "device_1": {
                "active": true,
                "error_msg": null,
                "error_timestamp": null,
                "last_update": null,
                "state": {
                    ...
                }
            }
        }
    }
}
```
- device_config：设备配置信息
- device_status：设备状态信息

## 接收：设备指令
Topic
```
cmd/{application_name}/{scenario_name}/deviceserver/{server_id}/{device_type}/{device_id}
```

Payload
```json
{
    ...
	"data":{
        "action":"on",
        "param":null
    }
}
```


## 接收：更新文件指令
Topic
```
broadcast/{application_name}/{scenario_name}
```
Payload
```json
{
	...
	"data":{
        "action":"update",
        "param": {
            "file_update_url" : "xxx",
            "device_update_url": "xxx"
        }
    }	
}
```
