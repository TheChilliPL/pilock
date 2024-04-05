package dev.thechilli.gpio4k.utils

actual fun sleep(millis: Int) {
    Thread.sleep(millis.toLong())
}
