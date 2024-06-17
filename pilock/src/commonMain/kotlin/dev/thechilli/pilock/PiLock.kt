//package dev.thechilli.pilock
//
//import dev.thechilli.gpio4k.keypad.Keypad
//import dev.thechilli.gpio4k.lcd.CharacterDisplay
//import dev.thechilli.gpio4k.utils.Event
//import dev.thechilli.gpio4k.utils.padCenter
//import dev.thechilli.gpio4k.utils.sleepMs
//
//class PiLockApp(
//    val lcd: CharacterDisplay,
//    val keypad: Keypad,
//) {
//    init {
//        require(lcd.rows == 4) { "LCD must have 4 rows" }
//        require(lcd.columns == 20) { "LCD must have 20 columns" }
//    }
//
//    val onBeforeUpdate: Event<Unit> = Event()
//    val onAfterUpdate: Event<Unit> = Event()
//
//    fun start() {
//        onBeforeUpdate.invoke(Unit)
//        lcd.initialize()
//        lcd.clearDisplay()
//        lcd.setCursor(1, 3)
//        lcd.print("Hello, PiLock!")
//        lcd.setCursor(2, 4)
//        lcd.print("Initializing")
//        onAfterUpdate.invoke(Unit)
//        sleepMs(1000)
//        onBeforeUpdate.invoke(Unit)
//        lcd.clearDisplay()
//    }
//
//    var currentInput = ""
//
//    fun update() {
//        onBeforeUpdate.invoke(Unit)
//
//        val input = keypad.readKeys()
//
//        if(input.isNotEmpty()) {
//            // Process input
//            if(input[0] in codeChars) {
//                if(currentInput.length < codeLength) {
//                    currentInput += input[0]
//                    buzz(BuzzerReason.OK)
//                } else {
//                    buzz(BuzzerReason.FAIL)
//                }
//            } else if(input[0] == '*') {
//                if(currentInput.isNotEmpty()) {
//                    currentInput = currentInput.dropLast(1)
//                    buzz(BuzzerReason.CANCEL)
//                } else {
//                    buzz(BuzzerReason.FAIL)
//                }
//            } else if(input[0] == '#') {
//                if(currentInput == code) {
//                    drawUnlockScreen()
//                    buzz(BuzzerReason.UNLOCKED)
//                    onAfterUpdate.invoke(Unit)
//                    sleepMs(3000)
//                    currentInput = ""
//                    return
//                } else {
//                    buzz(BuzzerReason.WRONG_CODE)
//                    currentInput = ""
//                }
//            }
//        }
//
//        drawMainScreen(currentInput)
//
//        onAfterUpdate.invoke(Unit)
//
//        sleepMs(100)
//    }
//
//    fun drawMainScreen(input: String) {
//        lcd.clearDisplay()
//        lcd.setCursor(0, 0)
//        lcd.print("Enter your code:")
//        lcd.setCursor(2, 0)
//        lcd.print(
//            (0..<codeLength)
//                .joinToString(" ") { i ->
//                    if(input.length > i) "#" else "_"
//                }
//                .padCenter(20)
//        )
//    }
//
//    val codeChars = "0123456789".toSet()
//    val code = "13245768"
//    val codeLength get() = code.length
//
//    enum class BuzzerReason {
//        OK,
//        CANCEL,
//        FAIL,
//        UNLOCKED,
//        WRONG_CODE
//    }
//
//    fun buzz(reason: BuzzerReason) {
//        // TODO
//        println("Buzzing: $reason")
//    }
//
//    fun drawUnlockScreen() {
//        lcd.clearDisplay()
//        lcd.setCursor(1, 0)
//        lcd.print("Unlocked!".padCenter(20))
//    }
//}
