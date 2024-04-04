package dev.thechilli.pilock

import dev.thechilli.gpio4k.gpio.GpioIOMode
import dev.thechilli.gpio4k.gpio.SysFsGpioPin
import dev.thechilli.gpio4k.utils.use

fun main() {
    println("Hello, World!")
    val pinId = 0
    println("Reserving pin $pinId")

    SysFsGpioPin(pinId).use { pin ->
        println("Checking mode")
        println("Mode: ${pin.mode}")
        if (pin.mode != GpioIOMode.OUTPUT) {
            println("Setting mode to OUTPUT")
            pin.setMode(GpioIOMode.OUTPUT)
        }
        var boolean = false
        for (i in 1..10) {
            println("Setting pin $pinId to $boolean")
            pin.write(boolean)
            Thread.sleep(1000)
            boolean = !boolean
        }
        println("Closing pin $pinId")
    }
}
