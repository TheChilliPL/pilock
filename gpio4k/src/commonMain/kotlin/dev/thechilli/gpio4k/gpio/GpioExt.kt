package dev.thechilli.gpio4k.gpio

import dev.thechilli.gpio4k.utils.sleepUs

fun Iterable<GpioPin>.resetAll(mode: GpioIOMode = GpioIOMode.INPUT) {
    forEach { it.reset(mode) }
}

fun Iterable<GpioPin>.setModeOfAll(mode: GpioIOMode) {
    forEach { it.setMode(mode) }
}

fun GpioPin.keepHigh(delayUs: Int = 1, block: () -> Unit) {
    this.write(true)
    sleepUs(delayUs)
    block()
    this.write(false)
    sleepUs(delayUs)
}

fun GpioPin.pulse(lengthUs: Int = 1, delayUs: Int = 1) {
    this.write(true)
    sleepUs(lengthUs)
    this.write(false)
    sleepUs(delayUs)
}
