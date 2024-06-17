
import dev.thechilli.gpio4k.gpio.GpioIOMode
import dev.thechilli.gpio4k.gpio.GpiodPin
import dev.thechilli.gpio4k.gpio.pulse
import dev.thechilli.gpio4k.lcd.DirectHD44780Driver
import dev.thechilli.gpio4k.utils.closingScope

fun main() = closingScope {
    // LCD pins
    val resetPin = GpiodPin(0, 15).autoClose().setActiveLow(true).setMode(GpioIOMode.OUTPUT)
    val rsPin = GpiodPin(0, 0).autoClose()
    val ePin = GpiodPin(0, 5).autoClose()

    // Consecutive pins for data D7–D0. Red, orange, yellow…
    var pinIds = listOf(17, 27, 22, 24, 10, 9, 11, 7)
    val use4Only = true
    if(use4Only) {
        for(pinId in pinIds.drop(4)) {
            val pin = GpiodPin(0, pinId).autoClose()
            pin.reset(GpioIOMode.OUTPUT)
            pin.write(true)
        }
        pinIds = pinIds.take(4)
    }
    val dataPins = pinIds.map {
        GpiodPin(0, it)
    }.autoCloseAll()

//    val lcd = DirectDOGM204Display(
//        resetPin,
////    val lcd = DirectHD44780Display(
//        rsPin,
//        null,
//        enPin,
//        dataPins,
//        4,
//        20
//    )
    resetPin.pulse(1, 1000)

    val driver = DirectHD44780Driver(
        DirectHD44780Driver.Pins(
            rsPin,
            ePin,
            dataPins,
        ),
        twoLineMode = true,
    )

    println("Initializing display…")

    driver.initialize()

    // Init
    val initBytes = byteArrayOf(
        0b00101010, // 4-bit data, RE=1, REV=0
//        0b00111010, // 8-bit data, RE=1, REV=0
        0b00001001, // 4-line
        0b00000110, // bottom view
        0b00011110, // BS1=1
        0b00101001, // 4-bit data, RE=0, IS=1
//        0b00111001, // 8-bit data, RE=0, IS=1
        0b00011011, // Internal OSC BS0=1 -> Bias 1/6
        0b01101110, // Follower control
        0b01010111, // Power control
        0b01110010, // Contrast set
        0b00101000, // 4-bit data, RE=0, IS=0
//        0b00111000, // 8-bit data, RE=0, IS=0
        0b00001111, // Display on
    )

    for (byte in initBytes) {
        driver.writeData(byte.toUByte(), rs = false)
    }

//    lcd.functionSet(dataLength8Bit = true, twoLines = true, font5x10 = false)
//    lcd.displayControl(true, true, false)

    println("Trying to display…")

    // Clear display
    driver.clearDisplay()
    for(char in "Hello checkpoint") {
        driver.writeData(char.code.toUByte(), true)
    }

//    val ledPin = SysFsPwmPin(0, 0).autoClose()
//
//    println("Starting LED")
//    ledPin.enable()
//
//    for(i in 0..100) {
//        ledPin.setRatio(0.01 * i)
//        sleepMs(100)
//    }
}
