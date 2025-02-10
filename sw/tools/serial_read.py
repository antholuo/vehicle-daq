import serial
import time
import datetime

ser = serial.Serial(
    port='COM3',
    baudrate=38400,
)

print(ser.name)

# ser.open()

data = "Hello from Python!\n"
data2 = "abcd\n"

while(True):
    print(datetime.datetime.now(), ser.readline())



ser.close()