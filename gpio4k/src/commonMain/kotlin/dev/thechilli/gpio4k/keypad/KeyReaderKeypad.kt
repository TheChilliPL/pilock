package dev.thechilli.gpio4k.keypad

import dev.thechilli.gpio4k.utils.KeyReader

class KeyReaderKeypad(
    val keys: List<List<Char>>,
    val keyReader: KeyReader,
) : Keypad {
    init {
        require(keys.isNotEmpty()) { "Keys must not be empty" }
        require(keys.all { it.size == keys[0].size }) { "All rows must be equal-sized" }
        require(keys.size == keys[0].size) { "Rows must match key reader rows" }
    }
    override fun initialize() { }

    override val rows: Int
        get() = keys.size
    override val columns: Int
        get() = keys[0].size

    override fun getKey(column: Int, row: Int): Char {
        return keys[row][column]
    }

    override fun readKeys(): List<Char> {
        val keyCode = keyReader.readKey() ?: return emptyList()
        val char = keyCode.toInt().toChar()

        if(keys.any { it.any{ it == char }})
            return listOf(char)

        return emptyList()
    }
}
