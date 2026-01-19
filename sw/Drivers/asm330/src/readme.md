# IMU Driver

## Dates
Anthony goes home Dec 20, when he will start trying to integrate IMU and GPS drivers
Drivers done like Dec 12-15
Sensor fusion for just IMU

Sensor Fusion
* Verifying sensor data (strap IMU to car)
* Data filtering
* Combining IMU and GPS data in real time


## Features
- [x] is_who_am_i_good
- [x] configure_spi
- [x] sw_reset
- [x] enable_accel
- [x] disable_accel
- [x] enable_gyro
- [x] disable_accel
- [ ] enable_gyro then accel
- [x] enable timestamp
- [x] single gyro, accel, timestamp reads
- [ ] configure_den
- [ ] block_reads
- [ ] accel offset
- [ ] fifo gyro, accel, timestamp
- [ ] fifo watermark
- [ ] den and fifo


## Driver Design
Here are the considerations that I would like to make.
* There are some fields that need to be exclusive
* Register field metadata (register fields and data restrictions) should be
    * stored in one place (single source of truth)
    * be easy to read and modify if needed
* As much configuration validation should be done at runtime as possible
* Configurations should be easy to make and verify

In pursuit of these goals, the following design is proposed
* Store register metadata in a rust file
* Configurations are created in a yaml file (most human friendly)
* Configurations are validated at compile time and written to generated source files
* Runtime configuration objects: list of static Config objects (immutable)
    * contains register writes sacrificing memory for configuration write speed
    * contains fields sacrificing memory for setting access
* Fields: used at runtime for reading, used at compile time for generating RegisterWrites
* At runtime, user should fetch and write configuration by index
* No runtime modification of fields

Register Field Metadata
* register address: u8
* field offset: u8
* field width: u8
* default value (default 0): u16
* leak below: bool
* leak to lower: bool
* readonly: bool

Field:
* members: same as register field metadata
* functions:
    * new(...)

RegisterWrite:
* members:
    * address
    * value
* functions:
    * new(address, value)

Config:
* members
    * reg_writes: list of RegisterWrites
    * fields: read only hashmap of field names and their values
* functions
    * new(reg_writes, fields)
    * write_config(self, spibus)
    * fields_from_yaml(yaml_path) -> list of field hashmaps
    * parse_reg_writes(list of fields) -> list of RegisterWrites

Asm330:
* members:
    * spibus: spi bus to read and write from
    * config_index: index of config object
* functions:
    * new(self, spibus, config_index): constructor
    * write_config(self): write config to spi bus
    * read_gyro_single(self): checks status and reads gyro output
    * read_accel_singel(self): checks status and reads accel output
    * read_time_single(self): checks status and reads timestamp output
    * read_temp_single(self): checks status and reads temperature output
    * empty_fifo(self): read all data from fifo
    * read_interrupt(self): read interrupt status
