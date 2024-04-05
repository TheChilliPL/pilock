package dev.thechilli.gpio4k.utils

internal actual fun checkDebug(): Boolean {
    return Platform.isDebugBinary
}
