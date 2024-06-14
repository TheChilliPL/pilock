package dev.thechilli.gpio4k.utils

fun String.padCenter(length: Int, padChar: Char = ' '): String {
    if (length <= this.length) return this
    val padSize = length - this.length
    val padStart = padSize / 2
    val padEnd = padSize - padStart
    return "${padChar.toString().repeat(padStart)}$this${padChar.toString().repeat(padEnd)}"
}
