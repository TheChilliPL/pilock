package dev.thechilli.gpio4k.lcd

import dev.thechilli.gpio4k.gpio.GpioIOMode
import dev.thechilli.gpio4k.gpio.GpioIOMode.INPUT
import dev.thechilli.gpio4k.gpio.GpioIOMode.OUTPUT
import dev.thechilli.gpio4k.gpio.GpioPin
import dev.thechilli.gpio4k.utils.bitFromRight
import dev.thechilli.gpio4k.utils.sleepMs
import dev.thechilli.gpio4k.utils.sleepUs
import kotlin.math.roundToInt

open class DirectDOGM204Display(
    protected val resetPin: GpioPin,
    protected val rsPin: GpioPin,
    protected val rwPin: GpioPin?,
    protected val enablePin: GpioPin,
    protected val dataPins: List<GpioPin>,
    rows: Int,
    columns: Int,
) : DOGM204Display {
    init {
        require(dataPins.size == 4 || dataPins.size == 8) { "Data pins must be 4 or 8" }
        require(rows in setOf(1, 2, 4)) { "Unsupported number of rows: $rows" }

        resetPin.setMode(OUTPUT)
        rsPin.setMode(OUTPUT)
        rwPin?.setMode(OUTPUT)
        enablePin.setMode(OUTPUT)
    }

    fun reset() {
        resetPin.write(true)
        sleepUs(200)
        resetPin.write(false)
        sleepMs(1)
    }

    var doubleHeightConfiguration: DOGM204Display.DOGM204DoubleHeightConfiguration = DOGM204Display
        .DOGM204DoubleHeightConfiguration.DOUBLE_SINGLE_SINGLE // TODO Check default
        protected set

    var bias: DOGM204Display.DOGM204Bias = DOGM204Display.DOGM204Bias.BIAS_1_3
        protected set

    var oscillatorFrequency = 540
        protected set

    var contrast = 0b11010.toFloat() / 0b11111
        protected set

    override val characterRom: HD44780CharacterSet
        get() = DOGM204Display.ROM_C

    override fun initialize() {
        if(is4BitMode) synchronize4Bit()
        // 00111010
        // 00001001
        extendedFunctionSet(false, false, true)
        // 00000110
//        entryModeSet(true, false)
        dataShiftDirection(true, false) // ?
        // 00011110
        doubleHeightBiasShift(doubleHeightConfiguration, bias.bs1, false)
        // 00111001
        // 00011011
        configureOscillatorFrequency(bias.bs0, oscillatorFrequency)
        // 01101110
        followerControl(true, 0b110)
        // 01010111
        val contrastAsByte = (contrast * 0b11111).roundToInt().toUByte()
        iconContrastControl(false, true, contrastAsByte)
        // 01110010
        contrastPreciseSet(contrastAsByte)
        // 00111000
        // 00001111
        displayControl(true, true, true)
    }

    protected fun synchronize4Bit() {
        // https://en.wikipedia.org/wiki/Hitachi_HD44780_LCD_controller#Mode_selection
        // Make sure to switch to 8-bit data length
        writeData8Bit(0b0011u)
        writeData8Bit(0b0011u)
        writeData8Bit(0b0011u)
        // Switch to 4-bit data length
        writeData8Bit(0b0010u)
    }

    override var columns: Int = columns
        protected set

    override var rows: Int = rows
        protected set

    val is4BitMode: Boolean = dataPins.size == 4

    override val readingAvailable = rwPin != null

    enum class AddressSpace {
        /**
         * Character Generator RAM
         */
        CGRAM,

        /**
         * Display Data RAM
         */
        DDRAM,

        /**
         * Segment RAM
         */
        SEGRAM,
    }

    var addressSpace = AddressSpace.DDRAM
        protected set

    override val currentlyInCgRam: Boolean
        get() = addressSpace == AddressSpace.CGRAM

    override var currentAddress: UByte = 0u
        set(value) {
            field = value and when(addressSpace) {
                AddressSpace.CGRAM -> 0b0011_1111u
                AddressSpace.DDRAM -> 0b0111_1111u
                AddressSpace.SEGRAM -> 0b0000_1111u
            }
        }

    override fun clearDisplay() {
        addressSpace = AddressSpace.DDRAM
        currentAddress = 0u
        super.clearDisplay()
    }

    override fun returnHome() {
        addressSpace = AddressSpace.DDRAM
        currentAddress = 0u
        super.returnHome()
    }

    override fun setDdRamAddress(address: UByte) {
        addressSpace = AddressSpace.DDRAM
        currentAddress = address
        super.setDdRamAddress(address)
    }

    override fun setCgRamAddress(address: UByte) {
        addressSpace = AddressSpace.CGRAM
        currentAddress = address
        super.setCgRamAddress(address)
    }

    override fun setSegRamAddress(address: UByte) {
        addressSpace = AddressSpace.SEGRAM
        currentAddress = address
        super.setSegRamAddress(address)
    }

    override fun cursorDisplayShift(displayShift: Boolean, right: Boolean) {
        if (!displayShift) {
            // Cursor shift
            if (right) {
                currentAddress++
            } else {
                currentAddress--
            }
        }
        super.cursorDisplayShift(displayShift, right)
    }

    override fun setSize(rows: Int, columns: Int) {
        require(rows in setOf(1, 2, 4)) { "Unsupported number of rows: $rows" }

        this.rows = rows
        this.columns = columns

        functionSet(!is4BitMode, rows % 2 > 1, font5x10)
        extendedFunctionSet(false, false, rows > 2) // TODO
    }

    override var font5x10: Boolean = false
        set(value) {
            field = value
            functionSet(!is4BitMode, rows % 2 == 0, font5x10)
        }

    override val getLineOffsets: List<UByte>
        get() = when (rows) {
            1 -> listOf(0x00u)
            2 -> listOf(0x00u, 0x40u)
            3 -> listOf(0x00u, 0x20u, 0x40u)
            4 -> listOf(0x00u, 0x10u, 0x20u, 0x30u)
            else -> throw IllegalArgumentException("Unsupported number of rows: $rows")
        }

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
        _cursorDirection = if (increment) CursorDirection.Right else CursorDirection.Left
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
        if (!displayOn) TODO("Off display is not implemented yet!")
        _cursorVisible = cursorOn
        _cursorBlink = cursorBlink
        super.displayControl(displayOn, cursorOn, cursorBlink)
    }

    private fun setDataPinsMode(mode: GpioIOMode) {
        dataPins.forEach { it.setMode(mode) }
    }

    protected var reBitOn = false
    protected var isBitOn = false

    override fun functionSetIs(
        dataLength8Bit: Boolean,
        twoLines: Boolean,
        font5x10: Boolean,
        specialRegisters: Boolean
    ) {
        super.functionSetIs(dataLength8Bit, twoLines, font5x10, specialRegisters)
        reBitOn = false
        isBitOn = specialRegisters
    }

    override fun functionSetRev(
        dataLength8Bit: Boolean,
        twoLines: Boolean,
        font5x10: Boolean,
        reverseDisplay: Boolean
    ) {
        super.functionSetRev(dataLength8Bit, twoLines, font5x10, reverseDisplay)
        reBitOn = true
    }

    fun ensureIsReBits(isBit: Boolean?, reBit: Boolean?) {
        if(isBit != null && isBit != isBitOn) {
            println("IS bit was $isBitOn, expected $isBit")
            functionSetIs(!is4BitMode, rows % 2 == 0, font5x10, isBit)
        }
        if(reBit != null && reBit != reBitOn) {
            println("RE bit was $reBitOn, expected $reBit")
            if (reBit) {
                functionSetRev(!is4BitMode, rows % 2 == 0, font5x10, false) // TODO
            } else {
                functionSetIs(!is4BitMode, rows % 2 == 0, font5x10, isBitOn)
            }
        }
    }

    override fun writeData(rs: Boolean, data: UByte, reBit: Boolean?, isBit: Boolean?) {
        ensureIsReBits(isBit, reBit)

        setDataPinsMode(OUTPUT)

        println("WRITING RS $rs DATA ${data.toString(2).padStart(8, '0')}")

        rwPin?.write(false)
        rsPin.write(rs)

        if (!is4BitMode) {
            writeData8Bit(data)
        } else {
            writeData4Bit(data)
        }
    }

    private fun writeData8Bit(data: UByte) {
        for ((i, pin) in dataPins.withIndex()) {
            pin.write(data.bitFromRight(i))
        }

        sleepUs(1)
        enablePin.write(true)
        sleepUs(1)
        enablePin.write(false)
        sleepUs(1500)
    }

    private fun writeData4Bit(data: UByte) {
        for ((i, pin) in dataPins.withIndex()) {
            pin.write(data.bitFromRight(i + 4))
        }

        sleepUs(1)
        enablePin.write(true)
        sleepUs(1)
        enablePin.write(false)
        sleepUs(1)

        for ((i, pin) in dataPins.withIndex()) {
            pin.write(data.bitFromRight(i))
        }

        sleepUs(1)
        enablePin.write(true)
        sleepUs(1)
        enablePin.write(false)
        sleepUs(1500)
    }

    override fun readData(rs: Boolean): UByte {
        if (is4BitMode) TODO("4-bit reading is not implemented yet!")

        // Make sure the pins are in input mode
        setDataPinsMode(INPUT)

        rwPin?.write(true)
        rsPin.write(rs)

        sleepMs(1)
        enablePin.write(true)
        sleepMs(1)
        var output: UByte = 0u
        for ((i, pin) in dataPins.withIndex()) {
            if (pin.read())
                output = output or (1u shl i).toUByte()
        }
        enablePin.write(false)
        sleepMs(2)

        return output
    }
}
