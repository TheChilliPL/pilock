package dev.thechilli.gpio4k

interface HD44780Display : CharacterDisplay {
    val readingAvailable: Boolean
    val currentAddress: UByte
    val currentlyInCgRam: Boolean

    val characterRom: CharArray

    override fun writeChar(char: Char) {
        // TODO Replace with a character table
        // TODO Implement \n
        writeData(true, char.code.toUByte())
    }

    override fun breakLine() {
        // TODO Read position or keep track of it!
        setDdRamAddress(0x40u)
    }

    fun readChar(): Char {
        return Char(readData(true).toInt())
    }

    override fun clearDisplay() {
        writeData(false, 0x01u)
    }

    override fun returnHome() {
        writeData(false, 0x02u)
    }

    fun entryModeSet(
        increment: Boolean,
        shift: Boolean
    ) {
        val data = 0x04u or (if (increment) 0x02u else 0u) or (if (shift) 0x01u else 0u)
        writeData(false, data.toUByte())
    }

    fun displayControl(
        displayOn: Boolean = true,
        cursorOn: Boolean = false,
        cursorBlink: Boolean = false
    ) {
        val data = 0x08u or (if (displayOn) 0x04u else 0u) or (if (cursorOn) 0x02u else 0u) or (if (cursorBlink) 0x01u else 0u)
        writeData(false, data.toUByte())
    }

    fun cursorDisplayShift(
        displayShift: Boolean,
        right: Boolean
    ) {
        val data = 0x10u or (if (displayShift) 0x08u else 0u) or (if (right) 0x04u else 0u)
        writeData(false, data.toUByte())
    }

    fun functionSet(
        dataLength8Bit: Boolean = true,
        twoLines: Boolean = true,
        font5x10: Boolean = false
    ) {
        val data = 0x20u or (if (dataLength8Bit) 0x10u else 0u) or (if (twoLines) 0x08u else 0u) or (if (font5x10) 0x04u else 0u)
        writeData(false, data.toUByte())
    }

    fun setCgRamAddress(address: UByte) {
        val data = 0x40u or (address and 0x3Fu).toUInt()
        writeData(false, data.toUByte())
    }

    fun setDdRamAddress(address: UByte) {
        val data = 0x80u or (address and 0x7Fu).toUInt()
        writeData(false, data.toUByte())
    }

    fun readBusyAndAddress(): UByte {
        return readData(false)
    }

    fun writeData(rs: Boolean, data: UByte)
    fun readData(rs: Boolean): UByte
}
