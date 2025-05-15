use std::fmt::{Debug, Formatter};
use std::fs::OpenOptions;
use std::sync::atomic::AtomicU8;
use log::debug;
use memmap2::{MmapOptions, MmapRaw};
use crate::clock::{ClockDriver, ClockSource, MashMode};
use crate::{GpioError, GpioResult};

pub struct RawClockDriver {
    mmap: MmapRaw,
    offset: u32,
}

impl RawClockDriver {
    const CLOCK_BASE: u32 = 0x3F101000;

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

    pub fn get_gp0() -> GpioResult<Self> {
        Self::with_offset(0x70)
    }

    pub fn get_gp1() -> GpioResult<Self> {
        Self::with_offset(0x78)
    }

    pub fn get_gp2() -> GpioResult<Self> {
        Self::with_offset(0x80)
    }

    pub fn get_pcm() -> GpioResult<Self> {
        Self::with_offset(0x98)
    }

    pub fn get_pwm() -> GpioResult<Self> {
        let mut pwm = Self::with_offset(0xA0)?;
        pwm.set_enabled(false)?;
        Ok(pwm)
    }

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

    pub fn divi_divf_to_divisor(divi: u16, divf: u16) -> GpioResult<f32> {
        if divi == 0 || divi > 0xFFF || divf > 0xFFF {
            return Err(GpioError::InvalidArgument);
        }

        let divisor = divi as f32 + (divf as f32 / 1024.0);
        Ok(divisor)
    }

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
    fn enabled(&self) -> GpioResult<bool> {
        let mmap = self.mmap.as_ptr() as *const u32;
        // CM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };

        let register_value = unsafe { register_ptr.read_volatile() };
        let value = (register_value >> 4) & 0b1;

        Ok(value != 0)
    }

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

    fn mash_mode(&self) -> GpioResult<MashMode> {
        let mmap = self.mmap.as_ptr() as *const u32;
        // CM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };

        let register_value = unsafe { register_ptr.read_volatile() };
        let value = (register_value >> 9) & 0b11;

        MashMode::from_index(value as u8)
    }

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

    fn source(&self) -> GpioResult<ClockSource> {
        let mmap = self.mmap.as_ptr() as *const u32;
        // CM_CTL register
        let register_ptr = unsafe { mmap.add(0x00 / 4) };

        let register_value = unsafe { register_ptr.read_volatile() };
        let value = register_value & 0b1111;

        ClockSource::from_index(value)
    }

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
