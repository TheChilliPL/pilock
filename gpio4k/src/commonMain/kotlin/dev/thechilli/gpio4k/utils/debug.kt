package dev.thechilli.gpio4k.utils

internal expect fun checkDebug(): Boolean

val isDebug: Boolean by lazy { checkDebug() }
