package dev.thechilli.gpio4k.utils

import platform.posix.usleep

actual fun sleep(millis: Int) {
    usleep((millis * 1000).toUInt())
}
