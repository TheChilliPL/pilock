package dev.thechilli.gpio4k

interface CharacterDisplay {
    /** Writes a string to the display. */
    fun write(str: String) {
        str.forEach { writeChar(it) }
    }

    /**
     * Writes a character to the display.
     */
    fun writeChar(char: Char)

    /**
     * Clears the display and sets the cursor to the home position.
     */
    fun clearDisplay()

    /**
     * Sets the cursor to the home position.
     */
    fun returnHome()

    /**
     * Sets whether the cursor should move to the right or left when a character is written.
     */
    var cursorDirection: CursorDirection

    /**
     * Sets whether the display should shift when a character is written.
     */
    var displayShift: Boolean

    /**
     * Sets whether the cursor should be visible.
     */
    var cursorVisible: Boolean

    /**
     * Sets whether the cursor should blink.
     */
    var cursorBlink: Boolean

    /**
     * Shifts the cursor to the left or right.
     */
    fun shiftCursor(direction: CursorDirection)

    /**
     * Shifts the display to the left or right.
     */
    fun shiftDisplay(direction: CursorDirection)

    /**
     * Moves the cursor to the specified position.
     */
    fun setCursor(row: Int, column: Int)

    val rows: Int
    val columns: Int

    /**
     * Checks whether the display is busy.
     */
    fun readBusyFlag(): Boolean

    /**
     * Reads the current address.
     */
    fun readAddress(): Int
}