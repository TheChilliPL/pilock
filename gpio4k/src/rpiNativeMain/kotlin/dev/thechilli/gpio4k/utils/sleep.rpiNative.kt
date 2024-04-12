package dev.thechilli.gpio4k.utils

import platform.posix.usleep

actual fun sleepMs(millis: Int) {
    usleep((millis * 1000).toUInt())
}

actual fun sleepUs(micros: Int) {
    usleep(micros.toUInt())
}
