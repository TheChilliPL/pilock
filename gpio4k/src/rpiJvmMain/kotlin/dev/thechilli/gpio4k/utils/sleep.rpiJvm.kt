package dev.thechilli.gpio4k.utils

actual fun sleepMs(millis: Int) {
    Thread.sleep(millis.toLong())
}

actual fun sleepUs(micros: Int) {
    Thread.sleep(0, micros)
}
