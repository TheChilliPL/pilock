//! Raw clock manager driver.

use std::fmt::{Debug, Formatter};
use std::fs::OpenOptions;
use memmap2::{MmapOptions, MmapRaw};
use crate::clock::{ClockDriver, ClockSource, MashMode};
use crate::{GpioError, GpioResult};

/// Raw clock manager driver.
/// 
/// The official documentation for the clock manager is quite limited, but some information was found
/// in [G. J. van Loo, “BCM2835 Audio & PWM clocks,” Feb. 2013](<https://www.scribd.com/doc/127599939/BCM2835-Audio-clocks>).
/// This documentation provides some more details of the clock manager in a similar processor of the
/// same family, and most of the information is still applicable to the BCM2711.
/// 
/// Each of the clocks has a `CM_CTL` register and a `CM_DIV` register. The first one is used to control the clock:
/// enable or disable it, set the source and mash mode. The second one is used to set the divisor as
/// a pair of values: 12 bits for the integer part (DIVI) and 12 bits for the fractional part (DIVF).
/// The clock should be disabled before changing anything to avoid lock-ups and glitches. Setting any
/// of these registers requires a password to be written, which is `0x5A` in the highest byte.
/// 
/// # General-purpose clocks
/// 
/// The official BCM2711 documentation mentions 3 general-purpose clocks — GP0, GP1, and GP2, at offsets
/// `0x70`, `0x78`, and `0x80` respectively. These offsets are relative to the base address of the clock manager,
/// which is `0x3F101000` in our case.
/// 
/// # PCM and PWM clocks
/// 
/// Additionally, the aforementioned BCM2835 audio & PWM clocks documentation mentions two crucial clocks
/// for our purposes: PCM clock (offset `0x98`) and PWM clock (offset `0xA0`).
/// 
/// For each clock, the `CM_CTL` register is at offset `0x00` and the `CM_DIV` register is at offset `0x04`.
pub struct RawClockDriver {
    mmap: MmapRaw,
    offset: u32,
}

impl RawClockDriver {
    /// The base address of the clock manager in the BCM2711.
    pub const CLOCK_BASE: u32 = 0x3F101000;

    /// Creates a new [RawClockDriver] instance with the specified offset from the clock base address.
    pub fn with_offset(offset: u32) -> GpioResult<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/mem")?;

        let mmap = MmapOptions::new()
            .offset(Self::CLOCK_BASE as u64 + offset as u64)
            .len(0x08)
            .map_raw(&file)?;

        Ok(RawClockDriver {
            mmap,
            offset,
        })
    }

    /// Creates a new [RawClockDriver] instance for the general-purpose clock 0.
    pub fn get_gp0() -> GpioResult<Self> {
        Self::with_offset(0x70)
    }
    
    /// Creates a new [RawClockDriver] instance for the general-purpose clock 1.
    pub fn get_gp1() -> GpioResult<Self> {
        Self::with_offset(0x78)
    }

    /// Creates a new [RawClockDriver] instance for the general-purpose clock 2.
    pub fn get_gp2() -> GpioResult<Self> {
        Self::with_offset(0x80)
    }

    /// Creates a new [RawClockDriver] instance for the PCM clock.
    /// 
    /// This clock is not mentioned in the official BCM2711 documentation. See [RawClockDriver] documentation for more details.
    pub fn get_pcm() -> GpioResult<Self> {
        Self::with_offset(0x98)
    }

    /// Creates a new [RawClockDriver] instance for the PWM clock.
    /// 
    /// This clock is not mentioned in the official BCM2711 documentation. See [RawClockDriver] documentation for more details.
    pub fn get_pwm() -> GpioResult<Self> {
        let mut pwm = Self::with_offset(0xA0)?;
        pwm.set_enabled(false)?;
        Ok(pwm)
    }

    /// Converts a divisor value (as a floating-point number) to a pair of integers representing
    /// the integer part (DIVI) and the fractional part (DIVF) for the clock manager registers.
    /// 
    /// It fails with [GpioError::InvalidArgument] if the divisor is invalid (e.g. 0).
    pub fn divisor_to_divi_divf(divisor: f32) -> GpioResult<(u16, u16)> {
        let int_part = divisor as u16;
        let frac_part = (divisor.fract() * 1024.0) as u16;

        // DIVI is in bits 23:12, so in total 12 bits, must not be 0
        if int_part > 0xFFF || int_part == 0 {
            return Err(GpioError::InvalidArgument);
        }

        // DIVF is in bits 11:0, so in total 12 bits
        if frac_part > 0xFFF {
            return Err(GpioError::InvalidArgument);
        }

        Ok((int_part, frac_part))
    }

    /// Converts a pair of integers representing the integer part (DIVI) and the fractional part (DIVF)
    /// to a floating-point divisor value.
    /// 
    /// It fails when DIVI or DIVF are out of bounds (DIVI must be in range [1, 4095] and DIVF must be in range [0, 4095]).
    pub fn divi_divf_to_divisor(divi: u16, divf: u16) -> GpioResult<f32> {
        if divi == 0 || divi > 0xFFF || divf > 0xFFF {
            return Err(GpioError::InvalidArgument);
        }

        let divisor = divi as f32 + (divf as f32 / 1024.0);
        Ok(divisor)
    }

    /// Returns the current busy state of the clock driver.
    pub fn get_busy(&self) -> GpioResult<bool> {
        let mmap = self.mmap.as_ptr() as *const u32;
        // CM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };

        let register_value = unsafe { register_ptr.read_volatile() };
        let value = (register_value >> 7) & 0b1;

        Ok(value != 0)
    }
}

impl Debug for RawClockDriver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RawClockDriver({:?}, {:#02x})", self.mmap.as_ptr(), self.offset)
    }
}

impl ClockDriver for RawClockDriver {
    /// Checks whether the clock is currently enabled, by reading the `ENAB` bit of the `CM_CTL` register.
    fn enabled(&self) -> GpioResult<bool> {
        let mmap = self.mmap.as_ptr() as *const u32;
        // CM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };

        let register_value = unsafe { register_ptr.read_volatile() };
        let value = (register_value >> 4) & 0b1;

        Ok(value != 0)
    }

    /// Sets the enabled state of the clock by modifying the `ENAB` bit of the `CM_CTL` register.
    fn set_enabled(&mut self, enabled: bool) -> GpioResult<()> {
        let mmap = self.mmap.as_mut_ptr() as *mut u32;
        // CM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };

        let mut register_value = unsafe { register_ptr.read_volatile() };
        // Password
        register_value |= 0x5A << 24;
        if enabled {
            register_value |= 0b1 << 4; // Set the enable bit
        } else {
            register_value &= !(0b1 << 4); // Clear the enable bit
        }
        unsafe { register_ptr.write_volatile(register_value) };

        Ok(())
    }

    /// Gets the current mash mode of the clock by reading the `MASH` bits of the `CM_CTL` register.
    fn mash_mode(&self) -> GpioResult<MashMode> {
        let mmap = self.mmap.as_ptr() as *const u32;
        // CM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };

        let register_value = unsafe { register_ptr.read_volatile() };
        let value = (register_value >> 9) & 0b11;

        MashMode::from_index(value as u8)
    }

    /// Sets the mash mode of the clock by modifying the `MASH` bits of the `CM_CTL` register.
    fn set_mash_mode(&mut self, mode: MashMode) -> GpioResult<()> {
        let mmap = self.mmap.as_mut_ptr() as *mut u32;
        // CM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };

        let mut register_value = unsafe { register_ptr.read_volatile() };
        // Password
        register_value |= 0x5A << 24;
        let value = mode.to_index();
        register_value &= !(0b11 << 9); // Clear the mash mode bits
        register_value |= (value as u32) << 9; // Set the new mash mode
        unsafe { register_ptr.write_volatile(register_value) };

        Ok(())
    }

    /// Gets the current clock source by reading the `SRC` bits of the `CM_CTL` register.
    fn source(&self) -> GpioResult<ClockSource> {
        let mmap = self.mmap.as_ptr() as *const u32;
        // CM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };

        let register_value = unsafe { register_ptr.read_volatile() };
        let value = register_value & 0b1111;

        ClockSource::from_index(value)
    }

    /// Sets the clock source by modifying the `SRC` bits of the `CM_CTL` register.
    fn set_source(&mut self, source: ClockSource) -> GpioResult<()> {
        let mmap = self.mmap.as_mut_ptr() as *mut u32;
        // CM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };

        let mut register_value = unsafe { register_ptr.read_volatile() };
        // Password
        register_value |= 0x5A << 24;
        let value = source.to_index();
        register_value &= !0b1111; // Clear the source bits
        register_value |= value; // Set the new source
        unsafe { register_ptr.write_volatile(register_value) };

        Ok(())
    }

    /// Gets the current divisor value by reading the `CM_DIV` register and converting it to a floating-point number.
    fn divisor(&self) -> GpioResult<f32> {
        let mmap = self.mmap.as_ptr() as *const u32;
        // CM_DIV register
        let register_ptr = unsafe { mmap.add(0x04 / 4) };
        let register_value = unsafe { register_ptr.read_volatile() };
        let divi = (register_value >> 12) & 0xFFF;
        let divf = register_value & 0xFFF;

        let divisor = RawClockDriver::divi_divf_to_divisor(divi as u16, divf as u16)?;
        Ok(divisor)
    }

    /// Sets the divisor value by modifying the `CM_DIV` register with the provided floating-point number.
    fn set_divisor(&mut self, divisor: f32) -> GpioResult<()> {
        let mmap = self.mmap.as_mut_ptr() as *mut u32;
        // CM_DIV register
        let register_ptr = unsafe { mmap.add(0x04 / 4) };
        let (divi, divf) = RawClockDriver::divisor_to_divi_divf(divisor)?;

        let mut register_value = unsafe { register_ptr.read_volatile() };
        // Password
        register_value |= 0x5A << 24;
        register_value &= !(0xFFF << 12); // Clear the DIVI bits
        register_value |= (divi as u32) << 12; // Set the new DIVI
        register_value &= !0xFFF; // Clear the DIVF bits
        register_value |= divf as u32; // Set the new DIVF
        unsafe { register_ptr.write_volatile(register_value) };

        Ok(())
    }
}
