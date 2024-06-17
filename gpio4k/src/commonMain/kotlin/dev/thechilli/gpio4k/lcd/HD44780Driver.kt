package dev.thechilli.gpio4k.lcd

import dev.thechilli.gpio4k.throwables.GpioCommunicationException
import dev.thechilli.gpio4k.utils.prefixedHexFormat

/**
 * Interface for an HD44780 driver.
 *
 * Driver implementations are low-level objects responsible for sending commands
 * and data to the display.
 *
 * For everyday use, consider using higher-level [HD44780Display] API instead.
 *
 * @see <a href="https://www.sparkfun.com/datasheets/LCD/HD44780.pdf">HD44780 datasheet</a>
 *
 * @throws GpioCommunicationException if an error occurs during communication with the display.
 */
abstract class HD44780Driver {
    /**
     * Whether the display supports 8-bit data length.
     *
     * Used for the function set command during initialization.
     *
     * For internal implementation, use [isEffectively8Bit] to take into account the [forceSingleData] flag.
     */
    open val dataLength8Bit: Boolean
        get() = true
    /**
     * Forces the driver to act as if the display is in the 8-bit mode.
     *
     * If `true`, every [writeData] and [readData] call will be done as if the display is in 8-bit mode.
     * This is used for initialization.
     */
    protected var forceSingleData = false

    protected val isEffectively8Bit: Boolean
        get() = dataLength8Bit || forceSingleData

    /**
     * Whether the display supports two-line mode.
     *
     * Used for the function set command during initialization.
     */
    abstract val twoLineMode: Boolean

    /**
     * Whether the display supports 5x10 font.
     *
     * Used for the function set command during initialization.
     */
    open val font5x10: Boolean
        get() = false

    /**
     * Initializes the driver appropriately.
     *
     * Any parameters required for initialization should be passed to the constructor.
     *
     * The default implementation initializes the display alone using the method that can be found in the datasheet.
     * If the driver requires additional initialization, you can override this method and do it before calling
     * `super.initialize()`.
     */
    open fun initialize() {
        syncInterfaceMode()
        clearDisplay()
        displayOnOffControl(displayOn = true, cursorOn = false, cursorBlink = false)
    }

    /**
     * Performs function set three times to ensure the display is in 8-bit mode; then, if necessary, switches to 4-bit mode.
     */
    protected fun syncInterfaceMode() {
        forceSingleData = true

        functionSet(dataLength8Bit = true, twoLineMode, font5x10)
        functionSet(dataLength8Bit = true, twoLineMode, font5x10)
        functionSet(dataLength8Bit = true, twoLineMode, font5x10)
        if(!dataLength8Bit)
            functionSet(dataLength8Bit, twoLineMode, font5x10)

        forceSingleData = false
    }

    /**
     * Clears the entire display and sets the cursor to the home position.
     */
    open fun clearDisplay() {
        writeData(0b0000_0001u)
    }

    /**
     * Sets the cursor to the home position.
     * Also returns the display to its original state if it was shifted.
     */
    open fun returnHome() {
        writeData(0b0000_0010u)
    }

    /**
     * Sets the cursor move direction and display shift.
     *
     * @param increment Whether the cursor should move to the right (`true`) or
     * left (`false`) after writing a character.
     * @param shift Whether the display should shift when a character is written.
     * If `true`, it will seem as if the cursor does not move.
     */
    open fun setEntryMode(
        increment: Boolean = true,
        shift: Boolean = false,
    ) {
        var data: UByte = 0b0000_0100u
        if (increment) data = data or 0b0000_0010u
        if (shift) data = data or 0b0000_0001u
        writeData(data)
    }

    /**
     * Sets the display control options.
     *
     * @param displayOn Whether the display should be on (`true`) or off (`false`).
     * @param cursorOn Whether the cursor should be visible.
     * @param cursorBlink Whether the cursor character should blink.
     */
    open fun displayOnOffControl(
        displayOn: Boolean,
        cursorOn: Boolean,
        cursorBlink: Boolean,
    ) {
        var data: UByte = 0b0000_1000u
        if (displayOn) data = data or 0b0000_0100u
        if (cursorOn) data = data or 0b0000_0010u
        if (cursorBlink) data = data or 0b0000_0001u
        writeData(data)
    }

    /**
     * Shifts the cursor or the display to the left or right.
     *
     * @param displayShift Whether the entire display should shift (`true`) or
     * just the cursor (`false`).
     * @param right Whether the shift should be to the right (`true`) or left (`false`).
     */
    open fun cursorDisplayShift(
        displayShift: Boolean,
        right: Boolean,
    ) {
        var data: UByte = 0b0001_0000u
        if (displayShift) data = data or 0b0000_1000u
        if (right) data = data or 0b0000_0100u
        writeData(data)
    }

    /**
     * Sets interface data length, number of display lines, and character font.
     *
     * Note that this function *will not* throw an exception if the display does
     * not support the specified configuration, unless the function set command
     * can't be sent at all, in which case a [GpioCommunicationException] will
     * be thrown.
     *
     * @param dataLength8Bit Whether the interface data length should be 8 bits (`true`) or 4 bits (`false`).
     * @param twoLines Whether the display should have two lines (`true`) or one line (`false`).
     * @param font5x10 Whether the character font should be 5x10 (`true`) or 5x8 (`false`).
     */
    open fun functionSet(
        dataLength8Bit: Boolean,
        twoLines: Boolean,
        font5x10: Boolean,
    ) {
        var data: UByte = 0b0010_0000u
        if (dataLength8Bit) data = data or 0b0001_0000u
        if (twoLines) data = data or 0b0000_1000u
        if (font5x10) data = data or 0b0000_0100u
        writeData(data)
    }

    /**
     * Sets the CGRAM address.
     *
     * @param address Address in the CGRAM to set. Must be in range 0x00–0x3F.
     */
    open fun setCgramAddress(address: UByte) {
        require(address <= 0b0011_1111u) {
            "CGRAM address must be in range 0x00–0x3F, is ${address.toHexString(prefixedHexFormat)}."
        }

        writeData((0b0100_0000u.toUByte() or (address and 0b0011_1111u)))
    }

    /**
     * Sets the DDRAM address.
     */
    open fun setDdramAddress(address: UByte) {
        // TODO Check what if trying to access address higher than 79 (maximum according to the datasheet)
        // ? require(address < 80) {
        //     "DDRAM address must be in range 0x00–0x4F, is ${address.toHexString(prefixedHexFormat)}."
        // }
        require(address <= 0b0111_1111u) {
            "DDRAM address must be in range 0x00–0x7F, is ${address.toHexString(prefixedHexFormat)}."
        }

        writeData((0b1000_0000u.toUByte() or (address and 0b0111_1111u)))
    }

    open fun readBusyFlagAndAddress(): Pair<Boolean, UByte> {
        val data = readData()
        val busyFlag = (data and 0b1000_0000u) != 0u.toUByte()
        val address = (data and 0b0111_1111u)
        return busyFlag to address
    }

    /**
     * Writes data byte to the display, with [rs] indicating the RS pin state,
     * saying whether the data is an instruction (`false`) or character (`true`).
     *
     * If RW pin is available, it should be set to `false` (write mode) within this method.
     *
     * @param data Data byte to write to the display. For GPIO implementations, it's in order of D7–D0.
     */
    abstract fun writeData(data: UByte, rs: Boolean = false)

    /**
     * Whether reading data from the display is available (supported and available in the current configuration).
     *
     * If `false`, [readData] should throw an [UnsupportedOperationException].
     */
    abstract val readingAvailable: Boolean

    /**
     * Reads data byte from the display, with [rs] indicating the RS pin state,
     * saying whether to read address and busy flag (`false`) or data (`true`).
     *
     * If RW pin is available, it should be set to `true` (read mode) within this method.
     *
     * @return Data byte read from the display. For GPIO implementations, it's in order of D7–D0.
     *
     * @throws UnsupportedOperationException if [readingAvailable] is `false`.
     */
    abstract fun readData(rs: Boolean = false): UByte
}
