package dev.thechilli.gpio4k.gpio

import dev.thechilli.gpio4k.utils.decodeToString
import dev.thechilli.gpio4k.utils.encodeToUByteArray

expect fun writeSysFs(path: String, value: UByteArray)
fun writeSysFs(path: String, value: String) = writeSysFs(path, value.encodeToUByteArray())

expect fun readSysFs(path: String): UByteArray
fun readSysFsString(path: String) = readSysFs(path).decodeToString().trim()
