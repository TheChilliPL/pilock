package dev.thechilli.gpio4k.buzzer

interface Buzzer {
    fun buzz(frequencyHz: UInt, durationMs: UInt)
}
