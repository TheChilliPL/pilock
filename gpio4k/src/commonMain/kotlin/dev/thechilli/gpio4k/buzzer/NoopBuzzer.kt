package dev.thechilli.gpio4k.buzzer

class NoopBuzzer : Buzzer {
    override fun buzz(frequencyHz: UInt, durationMs: UInt) {
        // noop
    }
}
