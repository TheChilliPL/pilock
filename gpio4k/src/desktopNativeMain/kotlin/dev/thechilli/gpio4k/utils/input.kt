package dev.thechilli.gpio4k.utils

import dev.thechilli.gpio4k.cinterop.conio.getch
import dev.thechilli.gpio4k.cinterop.conio.kbhit
import kotlinx.cinterop.alloc
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.ptr
import kotlinx.cinterop.value
import platform.windows.*

/**
 * Enable or disable input echo.
 */
fun setInputEcho(enabled: Boolean) = memScoped {
    val stdinHandle = GetStdHandle(STD_INPUT_HANDLE)
    val mode = alloc<DWORDVar>()

    GetConsoleMode(stdinHandle, mode.ptr)

    if (enabled) {
        mode.value = mode.value or ENABLE_ECHO_INPUT.toUInt()
    } else {
        mode.value = mode.value and ENABLE_ECHO_INPUT.toUInt().inv()
    }

    SetConsoleMode(stdinHandle, mode.value)
}

/**
 * Read a single character from the console.
 */
fun tryReadChar(): UByte? {
    if(kbhit() == 0) return null

    return getch().toUByte()
}
