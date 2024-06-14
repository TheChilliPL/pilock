
import dev.thechilli.gpio4k.keypad.KeyReaderKeypad
import dev.thechilli.gpio4k.lcd.MockHD44780CharacterDisplay
import dev.thechilli.gpio4k.utils.ConioKeyReader
import dev.thechilli.gpio4k.utils.KeyReader
import dev.thechilli.gpio4k.utils.closingScope
import dev.thechilli.gpio4k.utils.setInputEcho
import dev.thechilli.pilock.PiLockApp

fun main() = closingScope {
//    val buzzer = WindowsBuzzer()
//
//    val okMelody = Melody.of(
//        Note(440u, 50u),
//    )
//
//    val cancelMelody = Melody.of(
//        Note(440u, 50u),
//        Note(220u, 50u),
//    )
//
//    val npm = 100u
//    val quarterMs = 60000u / npm
//    val eighthMs = quarterMs / 2u
//    val sixteenthMs = quarterMs / 4u
//    val thirtySecondMs = quarterMs / 8u
//    val sixtyFourthMs = quarterMs / 16u
//    val failureMelody = Melody.of(
//        // Mario's death sound
//        Note(B4, sixteenthMs),
//        Note(0u, sixtyFourthMs),
//        Note(F5, thirtySecondMs * 3u / 2u),
//        Note(0u, sixteenthMs),
//
//        Note(F5, sixteenthMs),
//        Note(F5, eighthMs),
//        Note(E5, eighthMs),
//        Note(D5, eighthMs),
//
//        Note(C5, sixteenthMs),
//        Note(E4, sixteenthMs),
//        Note(G3, sixteenthMs),
//        Note(E4, eighthMs),
//        Note(C4, quarterMs * 3u / 2u),
//    )
//
//    buzzer.play(failureMelody)
//
//    return@closingScope

    println("Hello, PiLock Desktop Native!")
    setInputEcho(false)
    val keyReader = ConioKeyReader().autoClose().apply { initialize() } as KeyReader
    val keypad = KeyReaderKeypad(
        keys = listOf(
            listOf('1', '2', '3', 'A'),
            listOf('4', '5', '6', 'B'),
            listOf('7', '8', '9', 'C'),
            listOf('*', '0', '#', 'D'),
        ),
        keyReader = keyReader,
    )

    val display = MockHD44780CharacterDisplay(4, 20)

    val pilock = PiLockApp(display, keypad)

    pilock.onBeforeUpdate.subscribe {
        keyReader.update()
    }

    pilock.onAfterUpdate.subscribe {
        println("\u001B[1;1H\u001B[2J")
        display.printDisplayToConsole()
    }

    pilock.start()

    while(true) {
        pilock.update()
    }
}
