# !/bin/bash

# Caution: Please run as root since it is required to access /dev/* devices

# change this paramter to your output port path
MODBUS_PORT=/dev/ttyUSB0

echo "outout port: $MODBUS_PORT"

# port settings
stty -F $MODBUS_PORT 38400 cs8 -cstopb -parenb

# turn on the first coil
echo -ne '\x01\x05\x00\x01\xff\x00\xdd\xfa' > $MODBUS_PORT

sleep 2

# turn off the first coil
echo -ne '\x01\x05\x00\x01\x00\x00\x9c\x0a' > $MODBUS_PORT