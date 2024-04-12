package dev.thechilli.gpio4k.gpio

import kotlinx.cinterop.memScoped
import kotlinx.cinterop.refTo
import kotlinx.cinterop.toCValues
import platform.posix.*

actual fun writeSysFs(path: String, value: UByteArray) { memScoped {
    val fd = open(path, O_WRONLY)
    val cVal = value.toCValues()
    write(fd, cVal, value.size.toULong())
    close(fd)
} }

actual fun readSysFs(path: String): UByteArray {
    val fd = open(path, O_RDONLY)
    // TODO Support more bytes than buffer size
    val buffer = UByteArray(1024)
    val bytesRead = read(fd, buffer.refTo(0), buffer.size.toULong())
    close(fd)
    return buffer.copyOf(bytesRead.toInt())
}
