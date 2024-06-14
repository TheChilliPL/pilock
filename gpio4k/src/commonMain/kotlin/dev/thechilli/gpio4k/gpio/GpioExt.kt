package dev.thechilli.gpio4k.gpio

import dev.thechilli.gpio4k.utils.sleepUs

fun Collection<GpioPin>.resetAll(mode: GpioIOMode = GpioIOMode.INPUT) {
    forEach { it.reset(mode) }
}

fun GpioPin.keepHigh(delayUs: Int = 10, block: () -> Unit) {
    this.write(true)
    sleepUs(delayUs)
    block()
    this.write(false)
    sleepUs(delayUs)
}
