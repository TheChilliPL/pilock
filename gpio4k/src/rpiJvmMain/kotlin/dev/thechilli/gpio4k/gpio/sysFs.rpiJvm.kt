package dev.thechilli.gpio4k.gpio

import java.io.File

actual fun writeSysFs(path: String, value: UByteArray) {
    File(path).writeBytes(value.toByteArray())
}

actual fun readSysFs(path: String): UByteArray {
    return File(path).readBytes().toUByteArray()
}
