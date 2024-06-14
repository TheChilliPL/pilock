package dev.thechilli.gpio4k.keypad

interface Keypad {
    fun initialize()

    val rows: Int
    val columns: Int

    fun getKey(column: Int, row: Int): Char
    fun getKeyCoordinates(key: Char): Pair<Int, Int> {
        for (i in 0 until columns) {
            for (j in 0 until rows) {
                if (getKey(i, j) == key) {
                    return Pair(i, j)
                }
            }
        }
        throw NullPointerException("Key not found")
    }

    fun readKeys(): List<Char>

    fun isPressed(key: Char): Boolean = readKeys().contains(key)
}
