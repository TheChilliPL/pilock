package dev.thechilli.gpio4k.keypad

class MockKeypad(
    val keys: List<List<Char>>,
) : Keypad {
    init {
        require(keys.isNotEmpty()) { "Keys must not be empty" }
        require(keys[0].isNotEmpty()) { "Columns must not be empty" }
    }

    override fun initialize() {}

    override val rows: Int = keys.size
    override val columns: Int = keys[0].size

    private val pressedMap = BooleanArray(rows * columns)

    override fun getKey(column: Int, row: Int): Char = keys[row][column]

    override fun readKeys(): List<Char> {
        val keys = mutableListOf<Char>()

        for (i in 0 until columns) {
            for (j in 0 until rows) {
                if (pressedMap[j * columns + i]) {
                    keys.add(getKey(i, j))
                }
            }
        }

        return keys
    }

    fun mockKey(key: Char, pressed: Boolean) {
        val (column, row) = getKeyCoordinates(key)
        pressedMap[row * columns + column] = pressed
    }
}
