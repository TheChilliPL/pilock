package dev.thechilli.gpio4k.buzzer

import dev.thechilli.gpio4k.pwm.PwmPin
import dev.thechilli.gpio4k.utils.sleepMs

class PwmBuzzer(
    val pwmPin: PwmPin
) : Buzzer {
    override fun buzz(frequencyHz: UInt, durationMs: UInt) {
        if(frequencyHz == 0u) {
            sleepMs(durationMs.toInt())
            return
        }

        println("Beeping at $frequencyHz Hz for $durationMs ms")
        val periodNs = 1_000_000L / frequencyHz.toLong()

        pwmPin.setPeriodNs(periodNs)
        pwmPin.setRatio(0.5)

        pwmPin.enable()
        sleepMs(durationMs.toInt())
        pwmPin.disable()
    }
}
