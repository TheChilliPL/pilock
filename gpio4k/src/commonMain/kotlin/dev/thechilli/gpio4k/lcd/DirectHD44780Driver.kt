package dev.thechilli.gpio4k.lcd

import dev.thechilli.gpio4k.gpio.*
import dev.thechilli.gpio4k.utils.bitFromRight
import dev.thechilli.gpio4k.utils.sleepUs

class DirectHD44780Driver(
    val pins: Pins,
    override val twoLineMode: Boolean,
): HD44780Driver() {
    override val dataLength8Bit: Boolean
        get() = pins.data.size == 8
    override val readingAvailable: Boolean
        get() = pins.rw != null

    override fun initialize() {
        pins.resetAll()
        super.initialize()
    }

    private fun setDataPinsMode(mode: GpioIOMode) {
        pins.data.setModeOfAll(mode)
    }

    override fun writeData(data: UByte, rs: Boolean) {
        setDataPinsMode(GpioIOMode.OUTPUT)

        pins.rw?.write(false)
        pins.rs.write(rs)

        if(isEffectively8Bit) {
            writeDataRaw(data)
        } else {
            writeDataRaw((data.toInt() shr 4).toUByte())
            writeDataRaw(data and 0b1111u)
        }

        sleepUs(1500)
    }

    private fun writeDataRaw(data: UByte) {
        for ((index, pin) in pins.data.asReversed().withIndex()) {
            pin.write(data.bitFromRight(index))
        }
        sleepUs(1)

        pins.e.pulse()
    }

    override fun readData(rs: Boolean): UByte {
        setDataPinsMode(GpioIOMode.INPUT)

        pins.rw?.write(true)
        pins.rs.write(rs)

        pins.e.pulse(delayUs = 1500)

        return if(isEffectively8Bit) {
            readDataRaw()
        } else {
            val high = readDataRaw().toInt()
            val low = readDataRaw().toInt()
            ((high shl 4) or low).toUByte()
        }
    }

    private fun readDataRaw(): UByte {
        var output: UByte = 0u
        for ((index, pin) in pins.data.withIndex()) {
            if(pin.read())
                output = output or (1u shl index).toUByte()
        }
        return output
    }

    data class Pins(
        /** Register select pin. Low for command, high for character. */
        val rs: GpioPin,
        /** Enable pin. High pulse to execute. */
        val e: GpioPin,
        /** Data pins D7–D0 (or D7–D4 for 4-bit mode). */
        val data: List<GpioPin>,
        /** Read/write pin. Low for write, high for read. */
        val rw: GpioPin? = null,
    ) : Iterable<GpioPin> {
        init {
            require(data.size == 4 || data.size == 8) {
                "There must be 4 or 8 data pins, but there are ${data.size}."
            }
        }

        val all get() = setOf(rs, e) + setOf(rw).filterNotNull() + data

        override operator fun iterator() = all.iterator()
    }
}
