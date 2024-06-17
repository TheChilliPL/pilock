package dev.thechilli.gpio4k.gpio

import dev.thechilli.gpio4k.throwables.GpioException

/**
 * A GPIO pin that uses the sysfs interface.
 *
 * The API may or may not be deprecated (yes, it is), but it's simply the easiest way to access GPIO.
 * This class may get deprecated in the future as well.
 *
 * @throws
 */
class SysFsGpioPin(val pinId: Int) : GpioPin {
    val pinPath get() = "/sys/class/gpio/gpio$pinId"

    init {
        // Reserve the pin
        val exportPath = "/sys/class/gpio/export"
        try {
            writeSysFs(exportPath, pinId.toString())
        } catch (e: Exception) {
            throw GpioException("Failed to reserve pin $pinId", e)
        }

        reset()
    }

    override fun read(): Boolean {
        val valuePath = "$pinPath/value"
        if(mode != GpioIOMode.INPUT)
            throw GpioException("Pin $pinId is not readable")
        val value = readSysFsString(valuePath)
        return value == "1"
    }

    override fun write(value: Boolean) {
        val valuePath = "$pinPath/value"
        if(mode != GpioIOMode.OUTPUT)
            throw GpioException("Pin $pinId is not writable")
        writeSysFs(valuePath, if (value) "1" else "0")
    }

    override val mode: GpioIOMode
        get() {
            val directionPath = "$pinPath/direction"
            return when (val direction = readSysFsString(directionPath)) {
                "in" -> GpioIOMode.INPUT
                "out" -> GpioIOMode.OUTPUT
                else -> throw IllegalStateException("Invalid direction: $direction")
            }
        }

    override fun setMode(mode: GpioIOMode): SysFsGpioPin {
        val directionPath = "$pinPath/direction"
        writeSysFs(directionPath, when (mode) {
            GpioIOMode.INPUT -> "in"
            GpioIOMode.OUTPUT -> "out"
        })
        return this
    }

    override val activeLow: Boolean
        get() {
            val activeLowPath = "$pinPath/active_low"
            val activeLow = readSysFsString(activeLowPath)
            return activeLow == "1"
        }

    override fun setActiveLow(activeLow: Boolean): SysFsGpioPin {
        val activeLowPath = "$pinPath/active_low"
        writeSysFs(activeLowPath, if (activeLow) "1" else "0")
        return this
    }

    override val bias: GpioLineBias
        get() = GpioLineBias.NONE

    override fun setBias(bias: GpioLineBias): SysFsGpioPin {
        if(bias != GpioLineBias.NONE)
            throw UnsupportedOperationException("Bias is not supported by sysfs")
        return this
    }

    override val drive: GpioDriveMode
        get() = GpioDriveMode.PUSH_PULL

    override fun setDrive(drive: GpioDriveMode): SysFsGpioPin {
        if(drive != GpioDriveMode.PUSH_PULL)
            throw UnsupportedOperationException("Drive mode is not supported by sysfs")
        return this
    }

    override fun close() {
        // Release the pin
        val unexportPath = "/sys/class/gpio/unexport"
        writeSysFs(unexportPath, pinId.toString())
    }

    override fun toString(): String {
        return "SysFsGpioPin($pinId)"
    }
}
