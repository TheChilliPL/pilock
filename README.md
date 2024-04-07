# PiLock

PiLock is a simple and secure way to lock and unlock your door using a Raspberry Pi.

Originally a personal project, currently one for a university Embedded Systems course, in the future just open-source for anyone to use.

## Project setup

It's a multimodule project consisting of **PiLock** (the main module) and **GPIO4K** (a Kotlin library to interact with the Raspberry Pi GPIO pins, may be further separated in the future).

Each project is a multiplatform Kotlin project with the following source sets:

- `commonMain`: Shared code between all platforms.
  - `rpiCommonMain`: Shared code for all Raspberry Pi targets.
    - `rpiJvmMain`: Shared code for the JVM Raspberry Pi target. 
    - `rpiNativeMain`: Shared code for the Native Raspberry Pi target, which allows direct memory access to the GPIO pins.
  - `desktopJvmMain`: Shared code for the desktop JVM target, which is used for testing the shared code on a desktop environment.

## Features

### First to be implemented

- [ ] GPIO pin interaction
  - [x] Basic GPIO pin API
  - [ ] Debouncing
  - [ ] 4×4 keypad
  - [ ] Pulse-width modulation
- [x] HD44780 LCD interaction
- [ ] DOGM204x-A LCD interaction

### Next to be implemented

- [ ] Buzzer
- [ ] RFID reader
- [ ] Rotary encoder for the menu
- [ ] IR receiver for remote control

### Future features

- [ ] Home Assistant integration via MQTT
