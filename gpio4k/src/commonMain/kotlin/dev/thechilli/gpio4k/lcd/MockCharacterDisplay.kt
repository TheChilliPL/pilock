package dev.thechilli.gpio4k.lcd

class MockHD44780CharacterDisplay(
    override var rows: Int = 2,
    override var columns: Int = 16
) : HD44780Display {
    override fun initialize() {}

    override val getLineOffsets: List<UByte>
            get() = when (rows) {
                1    -> listOf(0x00u)
                2    -> listOf(0x00u, 0x40u)
                4    -> listOf(0x00u, 0x40u, 0x14u, 0x54u)
                else -> throw IllegalArgumentException("Unsupported number of rows: $rows")
            }

    override val readingAvailable: Boolean = true
    override var currentAddress: UByte = 0u
        private set
    private var currentShift: UByte = 0u
    override var currentlyInCgRam = false
        private set

    private val ddRam = UByteArray(0x80).apply { fill(' '.code.toUByte()) }

    override var characterRom = HD44780Display.ROM_A00

    var displayOn = true
    override var cursorDirection = CursorDirection.Right
    override var displayShift: Boolean = false
    override var cursorVisible: Boolean = true
    override var cursorBlink: Boolean = false // Not implemented

    override fun setSize(rows: Int, columns: Int) {
        this.rows = rows
        this.columns = columns
    }

    override var font5x10: Boolean
        get() = false
        set(_) { throw UnsupportedOperationException("5x10 font not supported") }

    private fun shiftCursorLeft() {
        currentAddress = (currentAddress.toByte() - 1).mod(0x80).toUByte()
    }

    private fun shiftCursorRight() {
        currentAddress = (currentAddress.toByte() + 1).mod(0x80).toUByte()
    }

    private fun shiftDisplayLeft() {
        currentShift = (currentShift.toByte() - 1).mod(0x80).toUByte()
    }

    private fun shiftDisplayRight() {
        currentShift = (currentShift.toByte() + 1).mod(0x80).toUByte()
    }

    override fun clearDisplay() {
        currentAddress = 0u
        currentlyInCgRam = false
        ddRam.fill(' '.code.toUByte())
    }

    override fun returnHome() {
        currentAddress = 0u
    }

    override fun entryModeSet(increment: Boolean, shift: Boolean) {
        cursorDirection = if (increment) CursorDirection.Right else CursorDirection.Left
        displayShift = shift
    }

    override fun displayControl(displayOn: Boolean, cursorOn: Boolean, cursorBlink: Boolean) {
        this.displayOn = displayOn
        cursorVisible = cursorOn
        this.cursorBlink = cursorBlink
    }

    override fun cursorDisplayShift(displayShift: Boolean, right: Boolean) {
        if (displayShift) {
            if(right) shiftDisplayRight() else shiftDisplayLeft()
        } else {
            if(right) shiftCursorRight() else shiftCursorLeft()
        }
    }

    override fun functionSet(dataLength8Bit: Boolean, twoLines: Boolean, font5x10: Boolean) {
        if(!twoLines) throw UnsupportedOperationException("Only two lines supported")
    }

    override fun setCgRamAddress(address: UByte) {
        currentAddress = address
        currentlyInCgRam = true
    }

    override fun setDdRamAddress(address: UByte) {
        currentAddress = address
        currentlyInCgRam = false
    }

    override fun setCursor(row: Int, column: Int) {
        currentAddress = (getLineOffsets[row] + column.toUByte()).toUByte()
    }

    override fun readBusyAndAddress(): UByte {
        return currentAddress and 0b0111_1111u // Busy flag is never on
    }

    override fun writeData(rs: Boolean, data: UByte) {
        if(!rs) {
            throw UnsupportedOperationException("Writing binary commands not supported")
        }
        if(currentlyInCgRam) {
            throw UnsupportedOperationException("Writing to CG RAM not supported")
        }
        ddRam[currentAddress.toInt()] = data
        if(displayShift) {
            if(cursorDirection == CursorDirection.Right)
                shiftDisplayRight()
            else
                shiftDisplayLeft()
        } else {
            if(cursorDirection == CursorDirection.Right)
                shiftCursorRight()
            else
                shiftCursorLeft()
        }
    }

    override fun readData(rs: Boolean): UByte {
        if(rs) {
            if(currentlyInCgRam)
                throw UnsupportedOperationException("Reading from CG RAM not supported")
            return ddRam[currentAddress.toInt()]
        } else {
            return readBusyAndAddress()
        }
    }

    fun printDisplayToConsole() {
        println("#".repeat(columns + 4))
        for (i in 0 until rows) {
            kotlin.io.print("# ")
            for (j in 0 until columns) {
                var index = getLineOffsets[i].toInt() + j
                index = index.mod(0x80)
                print(characterRom[ddRam[index]])
            }
            println(" #")
        }
        println("#".repeat(columns + 4))
    }
}
