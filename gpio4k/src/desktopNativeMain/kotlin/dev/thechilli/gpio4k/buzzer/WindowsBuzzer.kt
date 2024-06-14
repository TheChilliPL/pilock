package dev.thechilli.gpio4k.buzzer

import dev.thechilli.gpio4k.utils.sleepMs
import platform.windows.Beep

class WindowsBuzzer : Buzzer {
    override fun buzz(frequencyHz: UInt, durationMs: UInt) {
        println("Beeping at $frequencyHz Hz for $durationMs ms")
        if(frequencyHz != 0u)
            Beep(frequencyHz, durationMs)
        else
            sleepMs(durationMs.toInt())
    }
}
