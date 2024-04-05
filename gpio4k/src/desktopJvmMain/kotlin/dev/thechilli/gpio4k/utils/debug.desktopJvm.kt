package dev.thechilli.gpio4k.utils

internal actual fun checkDebug(): Boolean {
    return try { System.getProperty("debug") } catch (e: Exception) { null } != null
}
