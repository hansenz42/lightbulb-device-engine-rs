# 配置设备端口说明

部署设备服务时，需要将硬件上的设备端口与设备配置文件中的端口号做匹配，保证 device engine 能够正常访问端口。

而在 Debian 系统中，udev 服务管理硬件设备。alsa 服务管理音频设备。

## 1 使用 udev 配置设备端口

打开配置文件（如没有则新建）

```bash
vim /etc/udev/rules.d/99-local.rules
```

示例文件：

```
# MODBUS 接口
SUBSYSTEM=="tty", ENV{ID_PATH}=="pci-0000:00:14.0-usb-0:3:1.0", SYMLINK+="modbus0"
# Arduino USB 接口
SUBSYSTEM=="tty", ENV{ID_PATH}=="pci-0000:00:14.0-usb-0:4.1:1.0", SYMLINK+="arduino-usb1"
SUBSYSTEM=="tty", ENV{ID_PATH}=="pci-0000:00:14.0-usb-0:4.2:1.0", SYMLINK+="arduino-usb2"
SUBSYSTEM=="tty", ENV{ID_PATH}=="pci-0000:00:14.0-usb-0:4.3:1.0", SYMLINK+="arduino-usb3"
# FTDI DMX 接口
SUBSYSTEM=="tty", ENV{ID_PATH}=="pci-0000:00:14.0-usb-0:2.4:1.0", ENV{ID_MODEL}=="FT232R_USB_UART", SYMLINK+="ftdi0"

# 声卡接口
SUBSYSTEM=="sound", ACTION=="add", ENV{ID_PATH}=="pci-0000:00:14.0-usb-0:1.1:1.0", ATTR{id}="USB1"

# 剩下的 USB声卡 按照上述端口依次配置
```

说明
- SUBSYSTEM 为“子系统”
  - tty 的意思是字符终端。串口设备都是字符终端。我们使用的 MODBUS，Arduino，FTDI 都是基于串口的，所以都是 tty
  - sound 的意思是声卡
- ENV{ID_PATH} 指定了设备接入的路径，不同的 USB 接口该参数会不同。查看设备的 ID_PATH。关于如何查看设备端口路径，见章节 3。
- SYMLINK 表示设备软连接，设备接入后这里的配置设备会在 /dev/ 下面出现
- 配置完成后重启 udev 服务 systemctl restart systemd-udevd

## 2 使用 alsa 配置音频设备接口

使用 Linux alsa 服务的软件混音器，可以实现同一个声卡同时输出多个音频流

参考文件

```
pcm.usb1{
    type plug
    slave.pcm dmix:USB1
}
```

说明

- `pcm.usb1` 表示这是一个音频输出设备（pcm），命名为 usb1
- `type plug` 表示这是一个 ALSA plug 设备。plug 是一个虚拟设备，用来解决音频文件和声卡波特率不一致的问题
- `slave.pcm dmix:USB1` 表示 plug 设备后台接入的是 USB1 的软件混音器（dmix）。USB1 来自于设备端口配置，见章节 1
- 每一个 pcm plug 虚拟设备对应一个 USB 声卡，故如果有 6 张声卡，就应该有 6个 pcm plug 设备。所有 pcm 设备可以配置在同一个文件里
- 配置完成后保存文件即可，无需重启 ALSA 服务
- 如果功放直接接在工控机的音频输出口，则是：`slave.pcm dmix:PCH`

## 3 如何查看设备端口路径

根据 Linux 的设备管理逻辑，每次有新设备接入时，系统会自动分配一个设备 ID。但是，在部署 Linux 设备引擎时，我们会保证每个设备每次接入系统都在同一个路径，这样设备才能被正确识别。
每个 USB 设备在接入时有一个唯一地址，使用该地址配合 udev 我们就能固定每个设备在系统中的路径了。

- 我们了解，设备在接入时，Linux 内核会在 /dev/ 下生成一个文件描述符。但是这个文件描述符是不固定的。
  - 例如多个串口设备接入时，内核会按照接入的先后顺序给文件描述符起名字。例如三个 USB 串口 A B C。如果是以 B A C 的顺序接入插入接口，那么内核会按照 B=1 A=2 C=3 的顺序生成文件描述符。
- 所以，我们要将设备按照 USB 接口的位置固定下来。保证接口1插入的设备永远命名为1，接口2插入的设备命名为2，而不是以插入的先后顺序命名

### 获得 USB 端口位置

```bash
# 跳转到设备目录
cd /dev/

# 切换为 root
su root

# 列出所有设备描述符 
ls

>arduino-remote0  disk         hugepages     port    shm       tty16  tty3   tty43  tty57    ttyS1    vcsa   vga_arbiter
arduino-usb1     dri          initctl       ppp     snapshot  tty17  tty30  tty44  tty58    ttyS2    vcsa1  vhci
arduino-usb2     drm_dp_aux0  input         psaux   snd       tty18  tty31  tty45  tty59    ttyS3    vcsa2  vhost-net
arduino-usb3     fd           kmsg          ptmx    stderr    tty19  tty32  tty46  tty6     ttyUSB0  vcsa3  vhost-vsock
autofs           full         kvm           pts     stdin     tty2   tty33  tty47  tty60    ttyUSB1  vcsa4  watchdog
block            fuse         log           random  stdout    tty20  tty34  tty48  tty61    uhid     vcsa5  watchdog0
bsg              hidraw0      loop-control  rfkill  tty       tty21  tty35  tty49  tty62    uinput   vcsa6  zero
btrfs-control    hidraw1      mapper        rtc     tty0      tty22  tty36  tty5   tty63    urandom  vcsu
bus              hidraw2      mei0          rtc0    tty1      tty23  tty37  tty50  tty7     vcs      vcsu1
char             hidraw3      mem           sda     tty10     tty24  tty38  tty51  tty8     vcs1     vcsu2
console          hidraw4      modbus0       sda1    tty11     tty25  tty39  tty52  tty9     vcs2     vcsu3
core             hidraw5      mqueue        sda2    tty12     tty26  tty4   tty53  ttyACM0  vcs3     vcsu4
cpu              hidraw6      net           sda3    tty13     tty27  tty40  tty54  ttyACM1  vcs4     vcsu5
cpu_dma_latency  hidraw7      null          serial  tty14     tty28  tty41  tty55  ttyACM2  vcs5     vcsu6
cuse             hpet         nvram         sg0     tty15     tty29  tty42  tty56  ttyS0    vcs6     vfio
```

因为主要使用串口设备，所以关注 `ttyUSB*` 名字的设备。

```bash
# 接下来查看这几个设备的详细信息
udevadm info -n /dev/ttyUSB0

> P: /devices/pci0000:00/0000:00:14.0/usb1/1-3/1-3:1.0/ttyUSB0/tty/ttyUSB0
N: ttyUSB0
L: 0
S: modbus0
S: serial/by-id/usb-1a86_USB2.0-Serial-if00-port0
S: serial/by-path/pci-0000:00:14.0-usb-0:3:1.0-port0
E: DEVPATH=/devices/pci0000:00/0000:00:14.0/usb1/1-3/1-3:1.0/ttyUSB0/tty/ttyUSB0
E: DEVNAME=/dev/ttyUSB0
E: MAJOR=188
E: MINOR=0
E: SUBSYSTEM=tty
E: USEC_INITIALIZED=450042290791
E: ID_BUS=usb
E: ID_VENDOR_ID=1a86
E: ID_MODEL_ID=7523
E: ID_PCI_CLASS_FROM_DATABASE=Serial bus controller
E: ID_PCI_SUBCLASS_FROM_DATABASE=USB controller
E: ID_PCI_INTERFACE_FROM_DATABASE=XHCI
E: ID_VENDOR_FROM_DATABASE=Intel Corporation
E: ID_MODEL_FROM_DATABASE=Atom Processor Z36xxx/Z37xxx, Celeron N2000 Series USB xHCI
E: ID_PATH=pci-0000:00:14.0-usb-0:3:1.0
E: ID_PATH_TAG=pci-0000_00_14_0-usb-0_3_1_0
E: ID_VENDOR=1a86
E: ID_VENDOR_ENC=1a86
E: ID_MODEL=USB2.0-Serial
E: ID_MODEL_ENC=USB2.0-Serial
E: ID_REVISION=0262
E: ID_SERIAL=1a86_USB2.0-Serial
E: ID_TYPE=generic
E: ID_USB_INTERFACES=:ff0102:
E: ID_USB_INTERFACE_NUM=00
E: ID_USB_DRIVER=ch341
E: ID_MM_CANDIDATE=1
E: DEVLINKS=/dev/modbus0 /dev/serial/by-id/usb-1a86_USB2.0-Serial-if00-port0 /dev/serial/by-path/pci-0000:00:14.0-usb-0:3:1.0-port0
E: TAGS=:systemd:
E: CURRENT_TAGS=:systemd:
```

关注以上数据中的 `ID_PATH` 部分，即为 usb 设备接入地址，放入到 udev 配置文件中即可

说明
- ID_PATH 表示一个硬件设备的地址路径。`pci-0000:00:14.0-usb-0:3:1.0` 表示接在PCI总线上的USB控制总线的 `3:1.0` 接口上
- 插拔测试另外几个 USB 端口，可以发现只有 3:1.0 会变化。前面的字符串不会变化。所以我们在配置时主要关注 3:1.0 这一部分是否正确就可以了。
- 不同型号的工控机主板地址路径会不一样，最终的输出结果以 udevadm info 为准

## 附录

### 音频设备调试指令

查看音频设备：

```bash
# 查看所有的硬件设备
aplay -l

# 重看所有的 pcm 设备
aplay -L
```

播放一段音频：

```bash
aplay -Dplughw:0,0 somefile.wav
```

说明：
- `-D` 后面指定调试的音频设备
- 只能播放 wav 格式的文件 