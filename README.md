# PiLock

A smart electronic door lock built on Raspberry Pi, written in Rust, under the MIT license.

The repository is a Rust workspace divided into several packages:
- `pilock_gpio` (`./gpio`): GPIO interface for the Raspberry Pi. Home for most
    embedded functionality. Used as a dependency in the main program.
- `pilock_test_main` (`./test-main`): Test main program for the GPIO interface, used
    for testing. Will be removed in the future.
- `pilock` (`./pilock`): Main program for the PiLock project, which will be used to control the
    lock and manage the user interface.

## Building

We recommend cross-compiling PiLock on your development machine rather than building
it directly on the Raspberry Pi. It should be faster and easier to set up.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [cross](https://github.com/cross-rs/cross)

Install `cross` with:

```bash
cargo install cross
```

### Cross-compile for Raspberry Pi (64-bit)

To build for `aarch64`, run:

```bash
cross build --target aarch64-unknown-linux-gnu
```

This downloads the required toolchain and builds the binary. The resulting file
will be located at `target/aarch64-unknown-linux-gnu/debug/pilock`.

Keep in mind that the first build might take a while, as it needs to download
the toolchain and build all the dependencies from source. Subsequent builds will be faster.

### Deploy and run on Raspberry Pi

Transfer the binary to your Pi (e.g. using `scp` or the RustRover deployment tool):

```bash
scp target/aarch64-unknown-linux-gnu/debug/pilock <user>@<ip>:~/
```

Then SSH into your Pi and run the binary with debug logging enabled:

```bash
RUST_LOG=debug ./pilock
```

You can also use an `.env` file to set environment variables instead of specifying
them on the command line.

# Pinout

| Usage   | Function            |   Pin | Pin   | Function            | Usage   |
|---------|---------------------|------:|-------|---------------------|---------|
|         | 3V3                 |  1 🟨 | 🔴 2  | 5V                  |         |
| I2C SDA | GPIO 2 (I2C1 SDA)   |  3 🔵 | 🔴 4  | 5V                  |         |
| I2C SCL | GPIO 3 (I2C1 SCL)   |  5 🔵 | ⚫ 6   | GND                 |         |
|         | GPIO 4 (GPCLK0)     |  7 🟢 | 🟣 8  | GPIO 14 (UART TX)   |         |
|         | GND                 |   9 ⚫ | 🟣 10 | GPIO 15 (UART RX)   |         |
| LCD E¹  | GPIO 17             | 11 🟢 | 🟤 12 | GPIO 18 (PCM CLK)   |         |
| LCD RW¹ | GPIO 27             | 13 🟢 | ⚫ 14  | GND                 |         |
| LCD RS¹ | GPIO 22             | 15 🟢 | 🟢 16 | GPIO 23             |         |
|         | 3V3                 | 17 🟡 | 🟢 18 | GPIO 24             |         |
|         | GPIO 10 (SPI0 MOSI) | 19 🟠 | ⚫ 20  | GND                 |         |
|         | GPIO 9 (SPI0 MISO)  | 21 🟠 | 🟢 22 | GPIO 25             |         |
|         | GPIO 11 (SPI0 SCLK) | 23 🟠 | 🟠 24 | GPIO 8 (SPI0 CE0)   |         |
|         | GND                 |  25 ⚫ | 🟠 26 | GPIO 7 (SPI0 CE1)   |         |
|         | GPIO 0 (EEPROM SDA) | 27 🔵 | 🔵 28 | GPIO 1 (EEPROM SCL) |         |
|         | GPIO 5              | 29 🟢 | ⚫ 30  | GND                 |         |
|         | GPIO 6              | 31 🟣 | 🟢 32 | GPIO 12 (PWM0)      |         |
|         | GPIO 13 (PWM1)      | 33 🟣 | ⚫ 34  | GND                 |         |
|         | GPIO 19 (PCM FS)    | 35 🟤 | 🟢 36 | GPIO 16             | LCD D1¹ |
| LCD D0¹ | GPIO 26             | 37 🟢 | 🟤 38 | GPIO 20 (PCM DIN)   | LCD D2¹ |
|         | GND                 |  39 ⚫ | 🟤 40 | GPIO 21 (PCM DOUT)  | LCD D3¹ |

- 🟢 **GPIO** (General Purpose IO)
- 🟠 **SPI** (Serial Peripheral Interface)
- 🔵 **I2C** (Inter-Integrated Circuit)
- 🟣 **UART** (Universal Asynchronous Receiver-Transmitter)
- 🟤 **PCM** (Pulse Code Modulation)
- ⚫ **Ground**
- 🔴 **5V** (Power)
- 🟡 **3.3V** (Power)

¹ LCD is to be replaced with I2C or SPI protocol once implemented. That will free up these pins.
