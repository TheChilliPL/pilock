import dev.thechilli.gpio4k.gpio.GpiodPin
import dev.thechilli.gpio4k.lcd.DirectHD44780Display
import dev.thechilli.gpio4k.utils.closingScope

fun main() = closingScope {
    // LCD pins
    val rsPin = GpiodPin(0, 0).autoClose()
    val enPin = GpiodPin(0, 5).autoClose()

    // Consecutive pins for data
    val dataPins = listOf(17, 27, 22, 24, 10, 9, 11, 7).map {
        GpiodPin(0, it)
    }.asReversed().autoCloseAll()

    val lcd = DirectHD44780Display(
        rsPin,
        null,
        enPin,
        dataPins,
        4,
        20
    )

    // Init
    val initBytes = byteArrayOf(
        0b00111010, // 8-bit data, RE=1, REV=0
        0b00001001, // 4-line
        0b00000110, // bottom view
        0b00011110, // BS1=1
        0b00111001, // 8-bit data, RE=0, IS=1
        0b00011011, // Internal OSC BS0=1 -> Bias 1/6
        0b01101110, // Follower control
        0b01010111, // Power control
        0b01110010, // Contrast set
        0b00111000, // 8-bit data, RE=0, IS=0
        0b00001111, // Display on
    )

    for (byte in initBytes) {
        lcd.writeData(false, byte.toUByte())
    }

    // Clear display
    lcd.clearDisplay()
    lcd.print("Hello, World!")
}
