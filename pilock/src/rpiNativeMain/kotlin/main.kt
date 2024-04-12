
import dev.thechilli.gpio4k.gpio.GpiodPin
import dev.thechilli.gpio4k.lcd.DirectDOGM204Display
import dev.thechilli.gpio4k.pwm.SysFsPwmPin
import dev.thechilli.gpio4k.utils.closingScope
import dev.thechilli.gpio4k.utils.sleepMs

fun main() = closingScope {
    // LCD pins
    val resetPin = GpiodPin(0, 15).autoClose().setActiveLow(true)
    val rsPin = GpiodPin(0, 0).autoClose()
    val enPin = GpiodPin(0, 5).autoClose()

    // Consecutive pins for data
    val dataPins = listOf(17, 27, 22, 24, 10, 9, 11, 7).map {
        GpiodPin(0, it)
    }.autoCloseAll().asReversed()

    val lcd = DirectDOGM204Display(
        resetPin,
//    val lcd = DirectHD44780Display(
        rsPin,
        null,
        enPin,
        dataPins,
        4,
        20
    )

    println("Initializing display…")

    lcd.initialize()

    // Init
//    val initBytes = byteArrayOf(
//        0b00111010, // 8-bit data, RE=1, REV=0
//        0b00001001, // 4-line
//        0b00000110, // bottom view
//        0b00011110, // BS1=1
//        0b00111001, // 8-bit data, RE=0, IS=1
//        0b00011011, // Internal OSC BS0=1 -> Bias 1/6
//        0b01101110, // Follower control
//        0b01010111, // Power control
//        0b01110010, // Contrast set
//        0b00111000, // 8-bit data, RE=0, IS=0
//        0b00001111, // Display on
//    )
//
//    for (byte in initBytes) {
//        lcd.writeData(false, byte.toUByte())
//    }

//    lcd.functionSet(dataLength8Bit = true, twoLines = true, font5x10 = false)
//    lcd.displayControl(true, true, false)

    println("Trying to display…")

    // Clear display
    lcd.clearDisplay()
    lcd.print("Hello checkpoint")

    val ledPin = SysFsPwmPin(0, 0).autoClose()

    println("Starting LED")
    ledPin.enable()

    for(i in 0..100) {
        ledPin.setRatio(0.01 * i)
        sleepMs(100)
    }
}
