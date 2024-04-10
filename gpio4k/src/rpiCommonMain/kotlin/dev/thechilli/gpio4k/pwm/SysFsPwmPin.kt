package dev.thechilli.gpio4k.pwm

import dev.thechilli.gpio4k.gpio.GpioException
import dev.thechilli.gpio4k.gpio.readSysFsString
import dev.thechilli.gpio4k.gpio.writeSysFs

/**
 * A PWM pin using the sysfs interface.
 *
 * Unlike its GPIO counterpart, this API is not deprecated.
 *
 * - [Documentation](https://www.kernel.org/doc/Documentation/pwm.txt)
 */
class SysFsPwmPin(val chipId: Int, val channelId: Int) : PwmPin {
    constructor(channelId: Int): this(0, channelId)

    val pwmPath = "/sys/class/pwm/pwmchip$chipId/pwm$channelId"

    init {
        // Reserve the channel
        val exportPath = "/sys/class/pwm/pwmchip$chipId/export"
        try {
            writeSysFs(exportPath, channelId.toString())
        } catch (e: Exception) {
            throw GpioException("Failed to reserve channel $channelId", e)
        }

        reset()
    }

    override var enabled = false
        private set

    override fun enable() {
        writeSysFs("$pwmPath/enable", "1")
        enabled = true
    }

    override fun disable() {
        writeSysFs("$pwmPath/enable", "0")
        enabled = false
    }

    override val periodNs: Long
        get() {
            val periodPath = "$pwmPath/period"
            return readSysFsString(periodPath).toLong()
        }

    override fun setPeriodNs(periodNs: Long): SysFsPwmPin {
        writeSysFs("$pwmPath/period", periodNs.toString())
        return this
    }

    override val dutyCycleNs: Long
        get() {
            val dutyCyclePath = "$pwmPath/duty_cycle"
            return readSysFsString(dutyCyclePath).toLong()
        }

    override fun setDutyCycleNs(dutyCycleNs: Long): SysFsPwmPin {
        writeSysFs("$pwmPath/duty_cycle", dutyCycleNs.toString())
        return this
    }

    override val activeLow: Boolean
        get() {
            val polarityPath = "$pwmPath/polarity"
            return when(readSysFsString(polarityPath)) {
                "normal" -> false
                "inversed" -> true
                else -> throw IllegalStateException("Invalid polarity")
            }
        }

    override fun setActiveLow(activeLow: Boolean): SysFsPwmPin {
        writeSysFs("$pwmPath/polarity", if(activeLow) "inversed" else "normal")
        return this
    }

    override fun close() {
        // Unexport the channel
        val unexportPath = "/sys/class/pwm/pwmchip$chipId/unexport"
        writeSysFs(unexportPath, channelId.toString())
    }
}
