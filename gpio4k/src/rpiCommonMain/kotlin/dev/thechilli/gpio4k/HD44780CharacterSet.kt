package dev.thechilli.gpio4k

import kotlin.jvm.JvmInline

@JvmInline
value class HD44780CharacterSet private constructor(
    private val characters: CharArray
) {
    val size get() = characters.size

    operator fun get(index: UByte): Char = characters[index.toInt()]

    /**
     * Returns the index of the character in the character set.
     * If the character is not in the set, returns 00.
     */
    fun codeOf(char: Char): UByte = characters.indexOf(char).let {
        if (it == -1) 0u else it.toUByte()
    }

    operator fun contains(char: Char): Boolean = char in characters

    companion object {
        fun of(vararg characters: Char): HD44780CharacterSet = HD44780CharacterSet(characters)
    }
}
