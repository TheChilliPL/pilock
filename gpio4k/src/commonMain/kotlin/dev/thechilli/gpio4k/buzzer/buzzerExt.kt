package dev.thechilli.gpio4k.buzzer

fun Buzzer.play(note: Note) {
    buzz(note.frequencyHz, note.durationMs)
}

fun Buzzer.play(melody: Melody) {
    melody.notes.forEach {
        play(it)
    }
}
