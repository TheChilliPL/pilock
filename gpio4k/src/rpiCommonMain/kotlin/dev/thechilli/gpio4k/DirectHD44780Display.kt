package dev.thechilli.gpio4k

import dev.thechilli.gpio4k.DigitalState.HIGH
import dev.thechilli.gpio4k.DigitalState.LOW

/**
 * @param rsPin Register select pin.
 * @param rwPin Read/write pin. If null, the display is write-only.
 * @param enablePin Enable pin.
 * @param dataPins Data pins. The number of pins must be 4 or 8.
 * @param rows Number of rows on the display.
 * @param columns Number of columns on the display.
 * @param characterRom Character set of the display.
 */
class DirectHD44780Display(
        private val rsPin: DigitalOutput,
        private val rwPin: DigitalOutput?,
        private val enablePin: DigitalOutput,
        private val dataPins: List<DigitalOutput>,
        override val rows: Int,
        override val columns: Int,
        override val characterRom: HD44780CharacterSet = HD44780Display.ROM_A00,
) : HD44780Display {
    init {
        // Constructor parameter validation
        require(dataPins.size == 4 || dataPins.size == 8) { "Data pins must be 4 or 8" }
        require(rows in setOf(1, 2, 4)) { "Unsupported number of rows: $rows" }

        if(rwPin != null) {
            require(dataPins.all { it is DigitalInput }) { "For reading to be available, all data pins must be readable. If you don't want to read, set rwPin to null." }
        }
    }

    val is4BitMode: Boolean = dataPins.size == 4

    override val getLineOffsets: List<UByte> by lazy {
        when (rows) {
            1 -> listOf(0x00u)
            2 -> listOf(0x00u, 0x40u)
            4 -> listOf(0x00u, 0x40u, 0x14u, 0x54u)
            else -> throw IllegalArgumentException("Unsupported number of rows: $rows")
        }
    }

    override val readingAvailable: Boolean = rwPin != null

    override val currentAddress: UByte
        get() = if(readingAvailable) readData(false) else TODO("Reading is not available")

    override val currentlyInCgRam: Boolean
        get() = TODO("Not implemented yet!")

    private var _cursorDirection = CursorDirection.Right
    override var cursorDirection
        get() = _cursorDirection
        set(value) {
            entryModeSet(value == CursorDirection.Right, displayShift)
        }

    private var _displayShift = false
    override var displayShift
        get() = _displayShift
        set(value) {
            entryModeSet(cursorDirection == CursorDirection.Right, value)
        }

    override fun entryModeSet(increment: Boolean, shift: Boolean) {
        _cursorDirection = if(increment) CursorDirection.Right else CursorDirection.Left
        _displayShift = shift
        super.entryModeSet(increment, shift)
    }

    private var _cursorVisible = true
    override var cursorVisible
        get() = _cursorVisible
        set(value) {
            displayControl(displayOn = true, cursorOn = value, cursorBlink)
        }

    private var _cursorBlink = false
    override var cursorBlink
        get() = _cursorBlink
        set(value) {
            displayControl(displayOn = true, cursorOn = cursorVisible, value)
        }

    override fun displayControl(displayOn: Boolean, cursorOn: Boolean, cursorBlink: Boolean) {
        if(!displayOn) TODO("Off display is not implemented yet!")
        _cursorVisible = cursorOn
        _cursorBlink = cursorBlink
        super.displayControl(displayOn, cursorOn, cursorBlink)
    }

    override fun writeData(rs: Boolean, data: UByte) {
        if(is4BitMode) TODO("4-bit mode is not implemented yet!")

        rwPin?.write(LOW)
        rsPin.write(if(rs) HIGH else LOW)
        for((i, pin) in dataPins.withIndex()) {
            pin.write(if(data.bitFromLeft(i)) HIGH else LOW)
        }
    }

    override fun readData(rs: Boolean): UByte {
        TODO("Not yet implemented")
    }
}
